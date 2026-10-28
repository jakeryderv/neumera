# Design example: sorted vectors

This document works one demonstration end to end: sorted vectors of exact integers, defined and proved in Lean, exposed through Rust, and surfaced in Python. It is the concrete instance of the prototype scoped in [Bringing mathematical meaning into everyday programming](bringing-mathematical-meaning-into-everyday-programming.md). Its purpose is to make four things explicit before any general architecture is written:

1. The exact contracts, so the guarantees are strong enough to mean something.
2. The Rust surface, so the guarantees stay attached to actual data.
3. The mechanism that ties each Rust rule to the Lean contract it claims, so the two cannot silently drift.
4. Every trusted step, so a reader can see which code must be correct for the guarantees to hold.

Nothing here has been compiled. Lean names and signatures are written against core Lean 4 as understood at the time of writing and must be checked against the version pinned for the prototype.

## Why this example

Sortedness is a property that is O(n) to establish by inspection and O(1) to carry. Merging two sorted vectors produces a sorted vector, and a program that knows this avoids a re-scan. Binary search requires sortedness, and a program that can show provenance avoids both a re-scan and a silent wrong answer. So the example exercises every way evidence enters the system:

| Evidence source | Operation |
|---|---|
| Runtime construction | `sort` on arbitrary input |
| Runtime check | `isSorted` on arbitrary input |
| Transcribed theorem | `merge` of two sorted inputs |
| Requirement discharged by provenance | `binarySearch` |

Vector concatenation, the warm-up for the foreign function interface, carries no property worth tracking at runtime because its length is O(1) to read. It is still the first slice of the build (section 7), because it exercises the contract manifest with the simplest possible contract. It is not the demonstration.

## 1. Lean: definitions and contracts

The ordering relation is `≤` on `Int`, fixed once. Every definition and theorem below uses it, and nothing on the Rust or Python side reimplements it.

### Carrier

Theorems about sorting in core Lean are stated over `List`. The runtime representation that crosses the foreign function interface is `Array`. The prototype defines its operations over `Array Int` and proves its contracts by converting to `List` where the library theorems live. The conversion is total and the two views are provably equivalent, so this is bookkeeping rather than a gap.

### Predicates

```lean
namespace Neumera

/-- The one ordering relation used throughout. -/
abbrev le (a b : Int) : Prop := a ≤ b

/-- Sortedness: every element is ≤ every later element. -/
def Sorted (v : Array Int) : Prop := v.toList.Pairwise le

/-- Permutation: same elements with the same multiplicities. -/
def IsPerm (v w : Array Int) : Prop := v.toList.Perm w.toList

end Neumera
```

`List.Pairwise` and `List.Perm` are core Lean. `Pairwise le` is the standard way to state sortedness without Mathlib's `List.Sorted` alias, which keeps the Lean library free of a Mathlib dependency. That matters for build time and for the size of what ships.

### Operations

```lean
namespace Neumera

def sort (v : Array Int) : Array Int :=
  (v.toList.mergeSort (fun a b => decide (a ≤ b))).toArray

def merge (a b : Array Int) : Array Int :=
  (List.merge a.toList b.toList (fun x y => decide (x ≤ y))).toArray

/-- Adjacent-pair scan. Linear in the length. -/
def isSorted (v : Array Int) : Bool :=
  go v.toList
where
  go : List Int → Bool
  | [] => true
  | [_] => true
  | a :: b :: rest => decide (a ≤ b) && go (b :: rest)

/-- Index of some occurrence of `x`, if present. -/
def binarySearch (v : Array Int) (x : Int) : Option Nat := ...

end Neumera
```

`isSorted` is deliberately not the `Decidable` instance for `Pairwise`. That instance compares every element against every later one, which is quadratic. The adjacent scan is linear, and the contract below is what connects it back to `Sorted`.

`List.mergeSort` and `List.merge` exist in core Lean with correctness theorems. `binarySearch` is the one operation the prototype most likely implements and proves itself; core Lean has an `Array.binSearch` but its specification should be checked before relying on it.

### Contracts

Sortedness alone is not a contract. An implementation that returns an empty vector satisfies it. Each operation needs a sortedness clause and a content clause.

```lean
namespace Neumera

theorem sort_sorted (v : Array Int) : Sorted (sort v)
theorem sort_perm   (v : Array Int) : IsPerm (sort v) v

theorem merge_sorted (a b : Array Int) (ha : Sorted a) (hb : Sorted b) :
    Sorted (merge a b)
theorem merge_perm   (a b : Array Int) : IsPerm (merge a b) (a ++ b)

theorem isSorted_iff (v : Array Int) : isSorted v = true ↔ Sorted v

theorem binarySearch_some (v : Array Int) (x : Int) (hv : Sorted v) (i : Nat) :
    binarySearch v x = some i → ∃ h : i < v.size, v[i] = x
theorem binarySearch_none (v : Array Int) (x : Int) (hv : Sorted v) :
    binarySearch v x = none → x ∉ v.toList

theorem concat_length (a b : Array Int) : (a ++ b).size = a.size + b.size

end Neumera
```

Notes on each:

- `sort_sorted` and `sort_perm` follow from core's theorems about `mergeSort`, which require the comparison to be transitive and total. Both hold for `≤` on `Int`.
- `merge_sorted` and `merge_perm` follow from core's theorems about `merge`, under the same side conditions.
- `isSorted_iff` is what makes the runtime check trustworthy, and it is where the adjacent scan is connected to the pairwise predicate. The proof goes through the standard lemma that an adjacent chain implies pairwise under a transitive relation. Mathlib has that lemma; whether core does should be checked, and if not it is a short induction.
- `binarySearch_none` is the clause that gives a failed search meaning. Without it, `none` is indistinguishable from "gave up."
- Both search theorems take `Sorted v` as a hypothesis. That hypothesis is exactly what the Rust type carries, and it is the requirement discharged by provenance.
- `concat_length` is trivially cheap to observe at runtime. It exists so that the first implementation slice has a contract to put through the manifest.

### Exported surface

```lean
@[export neumera_sort]          def sortExport (v : Array Int) : Array Int := sort v
@[export neumera_merge]         def mergeExport (a b : Array Int) : Array Int := merge a b
@[export neumera_is_sorted]     def isSortedExport (v : Array Int) : Bool := isSorted v
@[export neumera_binary_search] def binarySearchExport (v : Array Int) (x : Int) : Option Nat :=
  binarySearch v x
@[export neumera_concat]        def concatExport (a b : Array Int) : Array Int := a ++ b
```

Each export is a one-line wrapper around the function the theorems are about. That one line is trusted (see section 5). Keeping exports trivial keeps the audit trivial.

## 2. Rust: types and surface

The Rust crate owns Lean objects and never copies vector contents into Rust unless the caller asks. This keeps `Int` honest: Lean's `Int` is arbitrary precision, and a fixed-width Rust integer is a different representation. Conversion to `Vec<i64>` is explicit and fallible.

### Ownership

Lean arrays are immutable and reference counted. All public operations below take `&self`. Handles are cheap to clone (a reference count increment), and the integration layer handles retaining and releasing Lean references internally. This gives Rust and Python identical semantics: an input is never consumed, so a caller can keep using it after an operation. The cost is that Lean cannot reuse a uniquely owned array in place; that optimization is irrelevant at prototype scale and can be revisited with measurements.

The `&self` surface and Lean's calling convention do not line up, and the binding layer has to bridge them. Lean's FFI documentation states that object parameters of `@[export]` functions are always **owned**: the callee consumes one reference and will release it. A Rust method holding only `&self` does not own a reference it can give away. So before passing an argument to an exported Lean function, the binding layer must retain an additional reference (`lean_inc_ref`), which Lean then consumes, leaving the Rust handle's own reference intact. Returned objects arrive owned by the caller and are released when the Rust handle drops. This convention must be confirmed against the pinned toolchain's FFI documentation during implementation. Getting it wrong is a use-after-free or a leak, not a type error, which is why row 5 of the trusted base is tested rather than assumed.

### Types

```rust
/// An immutable vector of exact integers, owned on the Lean heap.
/// No tracked properties.
pub struct IntVec {
    obj: LeanObj,            // owned `Array Int`; private
}

/// An `IntVec` that is known to be sorted under `≤`.
/// The field is private: there is no public way to build one except
/// through the operations below, and no public way to mutate the contents.
pub struct SortedIntVec {
    inner: IntVec,                       // private
    provenance: Option<Provenance>,      // private; see below
}

/// Diagnostic provenance: how sortedness was established, for inspection
/// and error messages. Not consulted for correctness, and not sufficient
/// for proof replay (it records neither inputs, results, nor theorem
/// versions). Optional because the tree grows with each merge.
pub enum Provenance {
    Sorted,                                   // sort_sorted
    Checked,                                  // isSorted_iff
    Merged(Box<Provenance>, Box<Provenance>), // merge_sorted
}
```

In v0 the knowledge language for this example is a single proposition, so the rule engine degenerates to Rust's own type checker applied to two wrapper types. That is the correct outcome for one property; the general engine appears only when properties multiply. Replayable evidence, which would need to identify actual inputs, results, and theorem versions, is explicitly deferred.

### Operations

```rust
impl IntVec {
    /// Build from Rust integers. Infallible: every i64 is an Int.
    pub fn from_slice(xs: &[i64]) -> IntVec;

    /// Copy out. Fails if any element exceeds i64.
    pub fn to_vec(&self) -> Result<Vec<i64>, Overflow>;

    pub fn len(&self) -> usize;

    /// Concatenation. Contract: concat_length. Calls neumera_concat.
    pub fn concat(&self, other: &IntVec) -> IntVec;

    /// Establish sortedness by construction.
    /// Contract: sort_sorted (and sort_perm for content). Calls neumera_sort.
    pub fn sort(&self) -> SortedIntVec;

    /// Establish sortedness by runtime check, O(n).
    /// Contract: isSorted_iff. Calls neumera_is_sorted.
    pub fn check_sorted(&self) -> Option<SortedIntVec>;
}

impl SortedIntVec {
    /// Contract: merge_sorted (and merge_perm for content). Calls neumera_merge.
    /// No re-check.
    pub fn merge(&self, other: &SortedIntVec) -> SortedIntVec;

    /// Requirement `Sorted v` discharged by the receiver's type.
    /// Contract: binarySearch_some, binarySearch_none. Calls neumera_binary_search.
    pub fn binary_search(&self, x: i64) -> Option<usize>;

    /// Read-only view. Cannot be used to mutate.
    pub fn as_int_vec(&self) -> &IntVec;

    /// Explicitly forget the property. Cheap: clones the handle.
    pub fn to_int_vec(&self) -> IntVec;

    pub fn provenance(&self) -> Option<&Provenance>;
}
```

Design rules the surface enforces:

- **No public constructor for `SortedIntVec`.** Only `sort`, `check_sorted`, and `merge` produce one.
- **No mutation.** `IntVec` is immutable; there is no `push`, `set`, or `as_mut_slice`. The property cannot be outlived because the contents cannot change.
- **Forgetting is explicit.** `to_int_vec` exists so a caller can hand a sorted vector to code that doesn't care. There is no implicit coercion.
- **The relation is defined once.** `check_sorted` calls Lean's `isSorted`, which `isSorted_iff` ties to the same `Sorted` the other theorems use. Rust never implements a comparison.
- **Fixed-width conversion is fallible.** `to_vec` can fail. This is the "mathematically honest representation" rule applied to the one place two representations meet.
- **Every guarantee-bearing rule names its contract.** `sort`, `check_sorted`, `merge`, and `binary_search` each reference a contract from the manifest (section 4). `concat` does too, for the first slice. `from_slice`, `to_vec`, and `len` bear no mathematical guarantee and reference nothing.

### Where the requirement is discharged

Binary search is only defined on `SortedIntVec`. A Rust caller holding an `IntVec` cannot call it; the compiler reports that the method does not exist. That is the requirement `Sorted v` discharged statically, by provenance encoded in the type. This is the one place in the demonstration where Rust's type checker, rather than a runtime check, does the work, and it is why the Python surface (section 3) must behave differently.

## 3. Python: classes and behavior

This prototype does not perform static checking of Python callers, so at the call site provenance is runtime data: the class of the object. The PyO3 adapter exposes the two Rust types as two Python classes with the same encapsulation. Because the Rust operations take `&self`, Python's retain-and-reuse semantics need no reconciliation; an operation never consumes its input in either language.

```python
from neumera import IntVec, SortedIntVec, MissingPropertyError

v = IntVec([3, 1, 2])
s = v.sort()                       # SortedIntVec, provenance "sorted"
t = IntVec([0, 5]).check_sorted()  # SortedIntVec, provenance "checked"
u = s.merge(t)                     # SortedIntVec, provenance "merged(sorted, checked)"
u.binary_search(5)                 # 4

v.binary_search(5)
# MissingPropertyError: binary_search requires a sorted vector.
#   This IntVec has no sortedness evidence.
#   Establish it with .sort() (O(n log n), reorders) or .check_sorted() (O(n), fails if unsorted).
```

Rules:

- `SortedIntVec` has no public `__init__`. Attempting to construct one directly raises.
- Neither class exposes item assignment or any mutating method. `list(v)` copies out; it does not alias.
- `check_sorted` on unsorted input raises rather than returning `None`, so that a forgotten check cannot be mistaken for a sorted result. The input is unaffected.
- `binary_search` on a plain `IntVec` raises `MissingPropertyError`. The message names the missing property and the operations that establish it. This is the "clear feedback when additional information is needed" requirement from the vision document, in its smallest form.
- `provenance` is exposed as a read-only nested structure for inspection, or `None` if history was not recorded.
- The two classes are distinct in the generated type stubs, so an ordinary Python type checker can distinguish them even though this prototype does not rely on that.

Python gets the same guarantees as Rust about the data, because the data lives in the same Lean objects behind the same Rust wrappers. What Python does not get is the static discharge; every requirement is checked at the call site by inspecting the class.

## 4. The contract manifest

Without an explicit mechanism, nothing connects a Rust rule to the theorem it claims. Delete `sort_sorted` from the Lean file and `IntVec::sort` still returns `SortedIntVec`, the Lean build still passes, and the Rust build still passes. The manifest closes that gap.

### What it checks and what it does not

The manifest checks that every guarantee-bearing Rust rule references a Lean contract that was checked in the same build, with the expected statement. It does not verify that the Rust wrapper applies the contract correctly. Whether `IntVec::sort` actually returns `SortedIntVec` only when `neumera_sort` was called, and whether `check_sorted` only wraps on a `true` result, remains trusted code. The manifest makes drift between Lean and Rust a build failure; it does not make the Rust rules correct.

It is one-directional. Every guarantee-bearing Rust rule must reference an available contract. A Lean contract with no Rust rule is allowed; the Lean library may prove more than the Rust surface uses.

### Lean side: the contracts module

A dedicated module restates each contract verbatim and discharges it by the named theorem.

```lean
namespace Neumera.Contracts

/-- export: neumera_sort -/
theorem sort_sorted : ∀ v : Array Int, Sorted (sort v) := Neumera.sort_sorted

/-- export: neumera_sort -/
theorem sort_perm : ∀ v : Array Int, IsPerm (sort v) v := Neumera.sort_perm

/-- export: neumera_is_sorted -/
theorem isSorted_iff : ∀ v : Array Int, isSorted v = true ↔ Sorted v := Neumera.isSorted_iff

/-- export: neumera_concat -/
theorem concat_length : ∀ a b : Array Int, (a ++ b).size = a.size + b.size :=
  Neumera.concat_length

-- and so on for merge_sorted, merge_perm, binarySearch_some, binarySearch_none
end Neumera.Contracts
```

If the underlying theorem is deleted, this module fails to elaborate. If its statement is weakened so that it no longer proves the restated contract, this module fails to elaborate. The restatement is the expected statement, written once, in Lean, where the kernel checks it.

### Axiom audit

Elaboration succeeding is not the same as the contract being proved. A proof replaced with `sorry` elaborates with a warning, and the contracts module above would still pass. The manifest emitter therefore collects the transitive axiom dependencies of every exported contract (the same information `#print axioms` shows) and rejects the build if any contract depends on `sorryAx` or on an axiom outside an explicit allowlist. The allowlist for this project is the three standard axioms: `propext`, `Classical.choice`, and `Quot.sound`. Anything else, including the axiom that `native_decide` introduces (`Lean.ofReduceBool` at the time of writing), fails the build until it is deliberately added to the allowlist with a reason. Tactic-related axiom names must be confirmed against the pinned Lean version when implementation begins. The axioms each contract depends on are recorded in the manifest so the Rust side and a reviewer can see them.

### What the manifest records

After the module elaborates and the audit passes, a Lean metaprogram run as part of the same `lake build` emits a manifest:

```json
{
  "lean_toolchain": "<pinned version>",
  "executable": "<hash of the compiled library this manifest describes>",
  "checked_artifacts": {
    "Neumera.Contracts": "<hash of the .olean>",
    "Neumera.Sorted": "<hash of the .olean>",
    "Neumera.Basic": "<hash of the .olean>"
  },
  "dependencies": "<hash of lake-manifest.json, pinning dependency revisions>",
  "axiom_allowlist": ["propext", "Classical.choice", "Quot.sound"],
  "contracts": [
    {
      "name": "Neumera.Contracts.sort_sorted",
      "statement": "∀ (v : Array Int), Sorted (sort v)",
      "export": "neumera_sort",
      "axioms": ["propext"]
    },
    {
      "name": "Neumera.Contracts.concat_length",
      "statement": "∀ (a b : Array Int), (a ++ b).size = a.size + b.size",
      "export": "neumera_concat",
      "axioms": []
    }
  ]
}
```

The export field comes from the doc comment or an attribute on each contract declaration.

Identity is recorded in three parts, and the executable hash alone is not enough. Proofs are erased from compiled code, so a proof-only change, including weakening a theorem and adjusting the contracts module to match, or swapping a proof for `sorry`, can leave the compiled library byte-identical. The `checked_artifacts` field identifies the Lean modules whose declarations were actually checked, by hash of their compiled `.olean` files, and `dependencies` pins the revisions of every Lean dependency those modules were built against. Together with the toolchain version, this is what makes "the same build" a checkable claim. The Rust build script does not merely record these values. Before generating any contract constant, it recomputes the hash of the library it is about to link, the hashes of the named `.olean` files, and the hash of the Lake dependency manifest, and compares each against the manifest. Any mismatch, or any named artifact that is missing, fails the build. A manifest from one build therefore cannot be paired with the library or proofs from another.

### Rust side: generated contract constants

The Rust build script reads the manifest, recomputes and compares the executable, checked-artifact, and dependency hashes as described above, checks that the toolchain version matches the pinned one, and only then generates one constant per contract:

```rust
// generated: contracts.rs
pub mod contracts {
    pub struct Contract { pub name: &'static str, pub statement: &'static str, pub export: &'static str }
    pub const SORT_SORTED: Contract = Contract {
        name: "Neumera.Contracts.sort_sorted",
        statement: "∀ (v : Array Int), Sorted (sort v)",
        export: "neumera_sort",
    };
    pub const CONCAT_LENGTH: Contract = Contract { /* ... */ };
    // ...
}
```

Each guarantee-bearing rule references its constant:

```rust
pub fn sort(&self) -> SortedIntVec {
    const _: &Contract = &contracts::SORT_SORTED;
    const _: &Contract = &contracts::SORT_PERM;
    // neumera_sort takes its argument owned and will release one reference.
    // Retain one for Lean to consume; `self.obj` keeps its own.
    let arg = self.obj.retain_for_owned_param();
    let obj = unsafe { LeanObj::from_owned(ffi::neumera_sort(arg)) };
    SortedIntVec { inner: IntVec { obj }, provenance: Some(Provenance::Sorted) }
}
```

If a contract is absent from the manifest, the constant is not generated, the reference does not resolve, and `rustc` fails. No separate checker is needed on the Rust side; the reference check is a compile error. A contract that no rule references is simply an unused constant.

The rule can also check that the export it calls matches the contract's export field. That is a cheap guard against attaching a theorem to the wrong operation, but it checks the name, not the behavior.

### Required negative test

The build must be shown to fail when a contract is removed or weakened. A test recipe (a `just` target and a CI job) does each of the following in a scratch copy and asserts that the build fails:

1. Delete `Neumera.sort_sorted`. Expected: the contracts module fails to elaborate.
2. Weaken `Neumera.sort_sorted` to a statement that no longer proves the restated contract. Expected: the contracts module fails to elaborate.
3. Delete the contract from the contracts module as well. Expected: the manifest omits it, the constant is not generated, and the Rust rule fails to compile.
4. Replace the manifest with one from a different build. Expected: the executable hash or checked-artifact check in the build script fails.
5. Replace the proof of `Neumera.sort_sorted` with `sorry`. Expected: Lean elaborates with a warning, the compiled library is unchanged, and the axiom audit fails the build on `sorryAx`.

Cases 3 and 5 matter most. Case 3 is the only way a determined edit can get past the Lean side's elaboration check. Case 5 is the one that the executable hash cannot catch, because the compiled output is identical with or without the proof.

### Relationship to generation

The manifest is the theorem-to-operation mapping made explicit. The "Generated" option in the vision document would emit the rule bodies from the same information. Building the manifest by hand first shows what that mapping has to contain.

## 5. The trusted base, enumerated

Each row is something that must be correct for the guarantees to hold. Nothing in this table has been built; "to be checked" means the build will check it once it exists.

| # | Step | Status | Audit surface |
|---|---|---|---|
| 1 | The theorems in section 1 | To be checked by Lean during build | None at runtime |
| 2 | The contracts module restates each contract as intended | To be checked by Lean for provability; the restatement matching intent is trusted | One declaration per contract |
| 3 | Each `@[export]` wraps the function its theorems are about | Trusted; the manifest records the association but does not check it | Five one-line definitions |
| 4 | Lean compiler and Lean runtime behave as specified | Assumed | Out of scope |
| 5 | Foreign function interface: initialization, symbol names, retaining a reference before each owned argument, releasing returned objects, error handling | Trusted, tested; scope to be set by the binding-layer evaluation | Binding layer plus the extern declarations |
| 6 | `IntVec::sort` returns `SortedIntVec` | References `sort_sorted` (manifest-checked); application trusted | One function |
| 7 | `IntVec::check_sorted` wraps iff `neumera_is_sorted` returned true | References `isSorted_iff` (manifest-checked); application trusted | One branch |
| 8 | `SortedIntVec::merge` returns `SortedIntVec` | References `merge_sorted` (manifest-checked); application trusted | One function |
| 9 | `SortedIntVec` has no public constructor and no mutation path | Trusted, enforced by Rust visibility | Struct definition and impl block |
| 10 | `from_slice` and `to_vec` convert correctly between `i64` and Lean `Int`, with overflow detected | Trusted, tested | Two functions |
| 11 | Manifest emitter (Lean) and consumer (Rust build script) agree on format; the consumer recomputes and compares executable, checked `.olean`, and dependency hashes and the toolchain version before generating constants | Trusted, tested by the negative tests in section 4 | Two small programs |
| 12 | Axiom audit collects transitive axioms per contract and rejects `sorryAx` and anything outside the allowlist | Trusted, tested by negative test 5; the allowlist itself is a reviewed configuration | Audit routine plus a three-entry allowlist |
| 13 | PyO3 classes mirror the Rust types without adding a constructor or mutation path | Trusted, tested | Adapter module |

Rows 6 through 8 are the transcribed rules. The manifest guarantees they reference real contracts; their bodies remain trusted. They are the only rows whose count grows with the number of theorems. Rows 3, 5, 9, 10, 11, 12, and 13 are fixed costs of the integration. Row 4 is the floor.

The document should be able to answer, for any guarantee a user relies on, which rows it rests on. For "this binary search result is correct": rows 1, 2, 3, 4, 5, then 6 or 7 or 8 depending on provenance, 9, 11, 12, and 10 if the result was read out as `i64`.

## 6. What to measure

The demonstration should report these, because they decide whether the approach is practical rather than merely sound.

- Clean build time, Lean and Rust together, on a developer machine and in CI.
- Wheel size, with the Lean runtime included.
- Foreign function call overhead for a trivial export, compared to a native Rust call.
- `merge` alone, versus the same `merge` followed by an independent `isSorted` scan, across sizes. This isolates the cost that preserving the guarantee saves. (Concatenating two sorted vectors and checking is not a valid comparison, since the concatenation is generally not sorted.)
- Time from editing a Lean theorem to a failing Rust build, and confirmation via the negative tests that the failure actually occurs.

## 7. The first implementation slice

The next step is not more prose. It is the smallest build that exercises every mechanism once:

1. **Pin a Lean version.** Record it in the repo. Check which of the theorems in section 1 exist in core at that version. Confirm the owned-argument convention for `@[export]` functions and the names of tactic-related axioms against that version's documentation.
2. **Evaluate the binding layer.** Try `lean-rs-sys` and `lean-rs` against the pinned version. Record what they cover (initialization, ownership, conversions, errors) and what the project would still write. Decide to depend, fork, or hand-write.
3. **Concatenation through Cargo.** Define `concat` and prove `concat_length` in Lean, export it, build with Lake under Cargo, call it from Rust, and read back the length.
4. **The manifest, end to end.** Contracts module with `concat_length`, axiom audit with the three-entry allowlist, manifest emitted by the same build recording executable hash, checked `.olean` hashes, dependency pins, and toolchain version, build script that checks identity and generates the constant, Rust rule that references it.
5. **The negative tests.** All five cases from section 4 fail the build as expected, including the `sorry` case that leaves the compiled library unchanged.
6. **Measurements.** Build time and call overhead from section 6, for this slice.

Concatenation's contract is trivial, and its length is cheap to observe at runtime, so this slice demonstrates no propagation saving. Its purpose is that every piece of machinery exists and is tested before the sortedness work adds the first interesting contract on top of it.

Open questions to settle during the slice rather than before it:

- `Array` versus `List` on the Lean side, if the conversion at each proof boundary turns out to be awkward.
- Whether the manifest's export association is carried as a doc comment or a custom attribute.
- Whether the checked-artifact identity is best taken from `.olean` hashes directly or from Lake's own trace files, which already encode much of the same information.

## 8. What this does and does not show

A working version of this document shows that Lean-proved guarantees can be attached to concrete data, carried through operations without re-checking, and used to discharge requirements, with the trusted base small enough to enumerate in a table and the Lean-to-Rust connection enforced by the build. It shows the three-language build working once.

It does not show symbolic reasoning, on-demand derivation, or more than one property interacting. Those are the next experiments, and the manifest and provenance structure here are where they would attach.
