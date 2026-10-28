# Mathematical Objects Cheat Sheet

Oct 27, 2026 · @Jake

## How to read this

Most mathematical objects are **a set plus structure**: a collection of elements, together with operations, relations, or a notion of nearness that obey certain rules (axioms). Each entry below gives the object, what it adds, and a standard example.

- **Carrier set** = the underlying elements (e.g. the integers).
- **Structure** = what you can do with them (add, compare, measure distance).
- **Axioms** = the rules the structure must satisfy (associativity, triangle inequality, ...).
- **Morphisms** = structure-preserving maps between objects of the same type (homomorphisms, continuous maps, linear maps).

## Foundations

Everything else is built from these.

| Object | What it is | Notation | Example |
| --- | --- | --- | --- |
| Set | Unordered collection of distinct elements | {a, b, c} | {1, 2, 3} |
| Multiset | Set where repeats count | {a, a, b} | Prime factors of 12: {2, 2, 3} |
| Tuple / ordered pair | Ordered list of fixed length | (a, b) | A point (3, 4) |
| Cartesian product | All ordered pairs from two sets | A × B | ℝ × ℝ = the plane |
| Relation | A subset of A × B saying which elements are related | a R b | "less than" on ℕ |
| Equivalence relation | Reflexive, symmetric, transitive relation; splits a set into classes | a \~ b | Congruence mod 5 |
| Function (map) | Assigns each input exactly one output | f : A → B | f(x) = x² |
| Injection / surjection / bijection | One-to-one / onto / both | ↪ / ↠ / ≅ | x ↦ 2x on ℝ is a bijection |
| Sequence | Function from ℕ to a set | (aₙ) | 1, 1/2, 1/3, ... |
| Cardinal | Size of a set | \|A\|, ℵ₀ | \|ℕ\| = ℵ₀ |
| Ordinal | Position in a well-ordering | ω, ω+1 | ω = order type of ℕ |

## Number systems

Each system extends the previous one to make a new operation always possible: ℕ ⊂ ℤ ⊂ ℚ ⊂ ℝ ⊂ ℂ.

| Set | Name | What it adds | Structure |
| --- | --- | --- | --- |
| ℕ | Natural numbers | Counting: 0 (or 1), 1, 2, ... | Commutative monoid |
| ℤ | Integers | Subtraction (negatives) | Commutative ring |
| ℚ | Rationals | Division (fractions p/q) | Field |
| ℝ | Reals | Limits (fills gaps like √2, π) | Complete ordered field |
| ℂ | Complex numbers | √−1 = i; every polynomial has a root | Algebraically closed field |
| ℍ | Quaternions | 4D numbers a + bi + cj + dk; models 3D rotations | Division ring (not commutative) |
| 𝕆 | Octonions | 8D numbers | Non-associative division algebra |
| ℤ/nℤ | Integers mod n | Clock arithmetic | Ring; a field when n is prime |
| ℚₚ | p-adic numbers | A different completion of ℚ, where nearness means divisible by high powers of p | Complete field |
| 𝔸 | Algebraic numbers | Roots of integer polynomials | Field |

## Algebraic structures

A set with one or more operations. Each row adds one axiom to the row above it.

**One operation**

| Structure | Adds | Example |
| --- | --- | --- |
| Magma | A binary operation (closure) | Integers under subtraction |
| Semigroup | Associativity | Positive integers under + |
| Monoid | Identity element | ℕ under +, with 0 |
| Group | Inverses | ℤ under +; rotations of a square |
| Abelian group | Commutativity | ℝ under + |

**Two operations (+ and ×)**

| Structure | Adds | Example |
| --- | --- | --- |
| Ring | Abelian group under +, monoid under ×, distributive law | ℤ; n × n matrices |
| Commutative ring | × is commutative | ℤ; polynomials ℝ\[x\] |
| Integral domain | No zero divisors (ab = 0 means a = 0 or b = 0) | ℤ |
| Division ring | Every nonzero element has a × inverse | Quaternions ℍ |
| Field | Commutative division ring | ℚ, ℝ, ℂ, ℤ/pℤ |

**Structures acting on other structures**

| Structure | What it is | Example |
| --- | --- | --- |
| Module | Abelian group with scalars from a ring | ℤⁿ over ℤ |
| Vector space | Module whose scalars form a field | ℝⁿ over ℝ |
| Algebra | Vector space that also has multiplication | n × n matrices over ℝ |
| Lie algebra | Vector space with a bracket \[x, y\] | 3D vectors under cross product |
| Group action | A group permuting a set's elements | Rotations acting on a cube |

Related objects: **subgroup**, **ideal** (the ring version of a normal subgroup), **quotient** (G/N, R/I), **homomorphism**, **isomorphism**.

## Linear algebra objects

| Object | What it is | Example |
| --- | --- | --- |
| Scalar | A single number from the field | 3 |
| Vector | Element of a vector space | (1, 2, 3) in ℝ³ |
| Basis | Minimal set of vectors that spans the space | {(1,0), (0,1)} for ℝ² |
| Linear map | Function preserving + and scalar × | Rotation of the plane |
| Matrix | Rectangular array; represents a linear map in a basis | 2 × 2 rotation matrix |
| Determinant | Number giving a square matrix's volume scaling | det(I) = 1 |
| Eigenvector / eigenvalue | v with Av = λv: a direction the map only stretches | Axis of a 3D rotation (λ = 1) |
| Inner product | Pairing ⟨u, v⟩ giving lengths and angles | Dot product |
| Dual space | All linear maps V → field | Row vectors vs column vectors |
| Tensor | Multilinear object; generalizes scalars (rank 0), vectors (1), matrices (2) | Stress tensor |
| Subspace / span / kernel / image | Sub-structures defined by a map or set of vectors | Kernel of a projection |

## Order structures

A set with a way of comparing elements (≤).

| Structure | Adds | Example |
| --- | --- | --- |
| Preorder | Reflexive and transitive | "Is reachable from" in a graph |
| Partial order (poset) | Antisymmetric: a ≤ b and b ≤ a means a = b | Subsets ordered by ⊆ |
| Total (linear) order | Every pair is comparable | ℝ under ≤ |
| Well-order | Every nonempty subset has a least element | ℕ under ≤ |
| Lattice | Every pair has a join (∨, least upper bound) and meet (∧, greatest lower bound) | Divisors of 12 under divisibility |
| Complete lattice | Every subset has a join and meet | All subsets of a set |
| Boolean algebra | Distributive lattice with complements; the algebra of logic | {true, false} with AND, OR, NOT |

## Spaces: topology and geometry

A set with a notion of nearness, distance, size, or shape. Read down the table to go from the weakest structure to the richest.

| Space | Adds | Example |
| --- | --- | --- |
| Topological space | Open sets: nearness without numbers | Any metric space; the Zariski topology |
| Hausdorff space | Distinct points can be separated by open sets | ℝⁿ |
| Metric space | Distance function d(x, y) | ℝⁿ with Euclidean distance |
| Complete metric space | Cauchy sequences converge (no holes) | ℝ (but not ℚ) |
| Normed vector space | Vector space with length ‖v‖ | ℝⁿ; continuous functions with max norm |
| Banach space | Complete normed space | Lᵖ spaces |
| Inner product space | Lengths and angles via ⟨u, v⟩ | ℝⁿ with dot product |
| Hilbert space | Complete inner product space | L² (used in quantum mechanics) |
| Manifold | Space that looks locally like ℝⁿ | Sphere, torus |
| Smooth manifold | Manifold where calculus works | Surface of the Earth |
| Riemannian manifold | Smooth manifold with lengths and angles on tangent vectors | Curved spacetime (Lorentzian variant) |
| Topological group | Group whose operations are continuous | Circle group |
| Lie group | Group that is also a smooth manifold | Rotations SO(3) |

**Geometric shapes** live inside these spaces: polygons, polyhedra, polytopes (n-dimensional), conic sections, curves, surfaces, knots, and fractals.

**Topological invariants** distinguish spaces: Euler characteristic, fundamental group, homology groups, genus (number of holes).

## Analysis objects

| Object | What it is | Example |
| --- | --- | --- |
| Limit | The value a sequence or function approaches | lim 1/n = 0 |
| Series | Infinite sum of a sequence | 1 + 1/2 + 1/4 + ... = 2 |
| Power series | Series in powers of x | eˣ = Σ xⁿ/n! |
| Derivative | Instantaneous rate of change | d/dx x² = 2x |
| Integral | Accumulated area or total | ∫₀¹ x dx = 1/2 |
| Differential equation | Equation relating a function to its derivatives | y′ = y |
| σ-algebra | Collection of sets that can be measured | Borel sets on ℝ |
| Measure | Assigns a size to sets | Lebesgue measure (length) |
| Measure space | Set + σ-algebra + measure | (ℝ, Borel, Lebesgue) |
| Probability space | Measure space with total measure 1 | A fair die |
| Random variable | Measurable function on a probability space | Outcome of the die roll |
| Function space | A space whose points are functions | C\[0,1\]; Lᵖ |
| Distribution (generalized function) | Object that acts on test functions | Dirac delta δ |
| Operator | Map between function spaces | Differentiation; Fourier transform |

## Discrete and combinatorial objects

| Object | What it is | Example |
| --- | --- | --- |
| Graph | Vertices joined by edges | A social network |
| Directed graph (digraph) | Edges have a direction | Web links |
| Weighted graph | Edges carry numbers | Road map with distances |
| Tree | Connected graph with no cycles | Family tree; file system |
| Hypergraph | Edges can join more than two vertices | Committees of people |
| Permutation | Rearrangement (bijection of a finite set) | (1 3 2) |
| Combination | Unordered selection | Choosing 3 of 10 people |
| Partition (of an integer) | Ways to write n as a sum | 4 = 3 + 1 = 2 + 2 = ... |
| Partition (of a set) | Split into disjoint nonempty blocks | {{1,2},{3}} |
| Matroid | Abstracts independence (from linear algebra and graphs) | Columns of a matrix |
| Combinatorial design | Arrangement with balance rules | Sudoku; finite projective plane |
| Formal language / automaton | Strings and machines that recognize them | Regular expressions |

## Category theory

The "math of math": it studies objects only through the maps between them.

| Object | What it is | Example |
| --- | --- | --- |
| Category | Objects + arrows (morphisms) that compose associatively, with identities | **Set** (sets and functions); **Grp** (groups and homomorphisms) |
| Functor | Structure-preserving map between categories | Forgetful functor **Grp** → **Set** |
| Natural transformation | Map between two functors | Double-dual V → V\*\* |
| Isomorphism | Arrow with a two-sided inverse | Bijection in **Set** |
| Limit / colimit | Universal "best fit" constructions | Products, pullbacks / coproducts, pushouts |
| Adjunction | Pair of functors in a universal relationship | Free group ⊣ forgetful functor |
| Monad | Endofunctor with unit and multiplication | Powerset monad; used in functional programming |
| Topos | Category that behaves like the category of sets | Sheaves on a space |
