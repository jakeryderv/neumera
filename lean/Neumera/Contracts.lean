import Neumera.Basic

/-!
# Contracts

Each declaration here restates, verbatim, a contract that a guarantee-bearing
Rust rule relies on, and discharges it by the named theorem in `Neumera.Basic`.

If the underlying theorem is deleted, this module fails to elaborate. If its
statement is weakened so that it no longer proves the restated contract, this
module fails to elaborate. The restatement is the expected statement, written
once, in Lean, where the kernel checks it.

The manifest emitter (`Manifest.lean`) reads this module. It requires every
theorem in this namespace to carry a doc comment of the form
`export: <symbol>`, naming the executable export the contract describes, and
it audits each theorem's transitive axioms against the allowlist.
-/


namespace Neumera.Contracts

/-- export: neumera_concat -/
theorem concat_length : ∀ a b : Array Int, (Neumera.concat a b).size = a.size + b.size :=
  Neumera.concat_length

end Neumera.Contracts
