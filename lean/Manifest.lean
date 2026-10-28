import Lean
import Neumera

/-!
# Contract manifest emitter

Run by the Rust build script after `lake build`, as a script:

```
lake env lean --run Manifest.lean <out.json> <static-lib> <lake-manifest.json> <lean-toolchain>
```

It loads the compiled environment, finds every theorem in `Neumera.Contracts`,
audits its transitive axioms against the allowlist, records its statement and
the executable export it describes, and writes a JSON manifest that identifies
the build: toolchain version, the hash of the static library, the hashes of
the checked `.olean` artifacts, and the hash of the Lake dependency manifest.

The Rust build script recomputes and compares every hash before generating
contract constants. The hash is FNV-1a 64 over file bytes: it detects drift
between a manifest and the artifacts it describes, and is not a cryptographic
commitment. The Rust side implements the same function.

Exit code 0 on success; 1 with a message on standard error otherwise.
-/

open Lean

namespace Neumera.Manifest

/-- The namespace whose theorems are the contracts. -/
def contractsNamespace : Name := `Neumera.Contracts

/-- The module prefix whose `.olean` files count as checked artifacts. -/
def projectPrefix : Name := `Neumera

/-- Axioms a contract may depend on. Anything else fails the build. The Rust
build script carries the same list and the two must agree. -/
def axiomAllowlist : Array Name := #[``propext, ``Classical.choice, ``Quot.sound]

/-- FNV-1a, 64-bit, over the bytes of a file. -/
def fnv1a64 (bytes : ByteArray) : UInt64 :=
  bytes.foldl (init := (0xcbf29ce484222325 : UInt64)) fun h b =>
    (h ^^^ b.toUInt64) * (0x100000001b3 : UInt64)

def hex16 (x : UInt64) : String :=
  let digits := String.ofList (Nat.toDigits 16 x.toNat)
  "".pushn '0' (16 - digits.length) ++ digits

def hashFile (path : System.FilePath) : IO String := do
  return hex16 (fnv1a64 (← IO.FS.readBinFile path))

/-- ASCII-trim a string (the `String` API returns a slice). -/
def trim (s : String) : String := s.trimAscii.toString

/-- Parse `export: <symbol>` from a contract's doc comment. -/
def parseExport? (doc : String) : Option String :=
  let doc := trim doc
  if doc.startsWith "export:" then
    let sym := trim (doc.drop "export:".length).toString
    if sym.isEmpty || sym.any Char.isWhitespace then none else some sym
  else none

structure ContractEntry where
  name : Name
  statement : String
  exportSymbol : String
  axioms : Array Name

def ContractEntry.toJson (c : ContractEntry) : Json :=
  Json.mkObj [
    ("name", Lean.toJson c.name.toString),
    ("statement", Lean.toJson c.statement),
    ("export", Lean.toJson c.exportSymbol),
    ("axioms", Lean.toJson (c.axioms.map (·.toString)))
  ]

def coreContext : Core.Context :=
  { fileName := "<manifest>", fileMap := FileMap.ofString "" }

/-- Pretty-print a type with the environment's notation. -/
def ppType (env : Environment) (type : Expr) : IO String := do
  let (fmt, _, _) ← (PrettyPrinter.ppExpr type).toIO coreContext { env }
  return toString fmt

def axiomsOf (env : Environment) (c : Name) : IO (Array Name) := do
  let (axs, _) ← (collectAxioms c : CoreM (Array Name)).toIO coreContext { env }
  return axs.qsort Name.lt

def fail {α} (msg : String) : IO α := throw (IO.userError msg)

/-- Every theorem declared directly in the contracts namespace, audited. -/
def collectContracts (env : Environment) : IO (Array ContractEntry) := do
  let some modIdx := env.getModuleIdx? contractsNamespace
    | fail s!"module {contractsNamespace} is not in the environment"
  let data := env.header.moduleData[modIdx.toNat]!
  let mut entries : Array ContractEntry := #[]
  for name in data.constNames do
    unless name.getPrefix == contractsNamespace && !name.isInternal do continue
    let some info := env.find? name
      | fail s!"contract {name} not found in environment"
    let .thmInfo _ := info
      | fail s!"contract {name} is not a theorem; contracts must be theorems"
    let some doc ← findDocString? env name
      | fail s!"contract {name} has no doc comment; expected `export: <symbol>`"
    let some exportSymbol := parseExport? doc
      | fail s!"contract {name}: doc comment must be exactly `export: <symbol>`, got: {trim doc}"
    let axioms ← axiomsOf env name
    for ax in axioms do
      if ax == ``sorryAx then
        fail s!"axiom audit failed: contract {name} depends on sorryAx (a proof is incomplete)"
      unless axiomAllowlist.contains ax do
        fail s!"axiom audit failed: contract {name} depends on axiom {ax}, which is not in the allowlist {axiomAllowlist}"
    let statement ← ppType env info.type
    entries := entries.push { name, statement, exportSymbol, axioms }
  if entries.isEmpty then
    fail s!"no contracts found in {contractsNamespace}"
  return entries

def checkToolchain (toolchainFile : System.FilePath) : IO String := do
  let line := trim (← IO.FS.readFile toolchainFile)
  -- `leanprover/lean4:v4.35.0` → `4.35.0`
  let pinned := match line.splitOn ":" with
    | [_, v] => if v.startsWith "v" then (v.drop 1).toString else v
    | _ => line
  unless pinned == Lean.versionString do
    fail s!"toolchain mismatch: {toolchainFile} pins {pinned} but the running Lean is {Lean.versionString}"
  return line

unsafe def runUnsafe (args : List String) : IO Unit := do
  let [outPath, libPath, lakeManifestPath, toolchainFile] := args
    | fail "usage: Manifest.lean <out.json> <static-lib> <lake-manifest.json> <lean-toolchain>"
  let toolchainLine ← checkToolchain toolchainFile
  initSearchPath (← findSysroot)
  -- Required for `loadExts`; this script only reads the environment.
  enableInitializersExecution
  -- `loadExts` so delaborators and other environment extensions are available.
  let env ← importModules #[{ module := projectPrefix }] {} 0 (loadExts := true)
  let contracts ← collectContracts env
  let modules := env.allImportedModuleNames.filter (projectPrefix.isPrefixOf ·) |>.qsort Name.lt
  let mut artifacts : List (String × Json) := []
  for m in modules do
    let olean ← findOLean m
    artifacts := artifacts ++ [(m.toString, Json.mkObj [
      ("path", Lean.toJson olean.toString), ("hash", Lean.toJson (← hashFile olean))])]
  let json := Json.mkObj [
    ("schema", Lean.toJson (1 : Nat)),
    ("hash_algorithm", Lean.toJson "fnv1a64"),
    ("lean_toolchain", Lean.toJson toolchainLine),
    ("lean_version", Lean.toJson Lean.versionString),
    ("lean_githash", Lean.toJson Lean.githash),
    ("executable", Json.mkObj [
      ("path", Lean.toJson libPath), ("hash", Lean.toJson (← hashFile libPath))]),
    ("checked_artifacts", Json.mkObj artifacts),
    ("dependencies", Json.mkObj [
      ("path", Lean.toJson lakeManifestPath), ("hash", Lean.toJson (← hashFile lakeManifestPath))]),
    ("axiom_allowlist", Lean.toJson (axiomAllowlist.map (·.toString))),
    ("contracts", Lean.toJson (contracts.map ContractEntry.toJson))
  ]
  IO.FS.writeFile outPath (json.pretty ++ "\n")
  IO.eprintln s!"manifest: {contracts.size} contract(s), {modules.size} checked module(s) → {outPath}"

@[implemented_by runUnsafe]
opaque run (args : List String) : IO Unit

end Neumera.Manifest

def main (args : List String) : IO UInt32 := do
  try
    Neumera.Manifest.run args
    return 0
  catch e =>
    IO.eprintln s!"manifest: error: {e}"
    return 1
