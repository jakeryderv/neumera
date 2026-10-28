/-!
# Basic definitions

Executable operations over immutable vectors of exact integers, together with
the theorems the Rust integration layer relies on. Each theorem that Rust
relies on is restated verbatim in `Neumera.Contracts` and discharged by the
theorem here; see that module and the contract manifest.
-/

namespace Neumera

/-- Vector concatenation. The warm-up operation for the foreign function
interface: its length contract is trivial, which makes it the right first
contract to put through the manifest machinery. -/
def concat (a b : Array Int) : Array Int := a ++ b

/-- The length of a concatenation is the sum of the input lengths. -/
theorem concat_length (a b : Array Int) : (concat a b).size = a.size + b.size := by
  simp [concat]

/-! ## Exported surface

Each export is a one-line wrapper around the function the theorems are
about. Keeping exports trivial keeps the audit trivial. All object
parameters of `@[export]` functions are owned: the callee consumes one
reference to each argument. -/

@[export neumera_concat]
def concatExport (a b : Array Int) : Array Int := concat a b

end Neumera
