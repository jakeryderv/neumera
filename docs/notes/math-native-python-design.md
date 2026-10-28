# Math-Native Python: Starting Design Reference

## Guiding idea

Represent mathematical meaning explicitly, while delegating computation to existing libraries.

Keep four questions separate:

1. **Domain:** Which objects or values are allowed?
2. **Structure:** Which properties, relations, operations, and laws apply?
3. **Interpretation:** What do the values represent?
4. **Representation:** How are they stored and computed?

An additional check is **invariance**: which conclusions survive a valid change of units, labels, or representation?

These are cooperating responsibilities, not levels of one inheritance tree. Start with concrete examples and extract abstractions as needed.

## Minimal package structure

`mathkit` is a placeholder package name.

```text
project/
├── pyproject.toml
├── README.md
├── src/
│   └── mathkit/
│       ├── __init__.py
│       ├── domains.py
│       ├── objects.py
│       ├── properties.py
│       ├── relations.py
│       └── operations.py
├── tests/
└── examples/
```

| Module | Responsibility | Initial examples |
|---|---|---|
| `domains.py` | Membership and allowed values | Integers, positive reals, finite categories |
| `objects.py` | Values and their attached meaning | A value with a domain; later, a length quantity |
| `properties.py` | Conditions on one object | Positive, finite, nonzero |
| `relations.py` | Connections or comparisons between objects | Equal, less than, compatible units |
| `operations.py` | Produce results from inputs | Addition, subtraction; later, summaries |

Keep `__init__.py` small: expose selected public names rather than implementing behavior there.

## Implementation order

### 1. Domains and objects — build together

Start with a `Domain` class holding a name and a membership check. Create one simple value class holding data and a reference to its domain.

Decide explicitly whether membership checks examine raw stored data or your wrapped objects; use one convention consistently.

Begin with integer values. Define how construction handles invalid inputs. Validation should not silently round or convert an invalid input into a valid one.

### 2. Properties

Add ordinary functions checking one object, such as `is_positive(value)`.

Distinguish:

- **Attributes:** stored descriptions such as data, domain, unit, or shape.
- **Mathematical properties:** conditions such as positivity or finiteness.
- **Python `@property`:** a mechanism for attribute access, not a mathematical classification.

### 3. Relations

Add functions such as `equal(a, b)`, `less_than(a, b)`, and eventually `compatible_units(a, b)`.

A property can be understood as a unary relation. Separate modules make their practical roles easy to find; they are not mutually exclusive mathematical categories.

### 4. Operations

Implement a few operations with an explicit sequence:

```text
Inspect inputs
    → check domains and required conditions
    → determine the output domain and meaning
    → calculate
    → construct the result
```

Do not simply copy input metadata. For example, the mean of counts may not be an integer, and subtracting two temperature readings produces a temperature difference.

### 5. Expand through contrasting examples

1. **Integers:** membership, arithmetic, and result domains.
2. **Length:** units, compatibility, conversion, and derived quantities.
3. **Temperature:** readings versus differences and offset conversions.
4. **Ordered categories:** meaningful ordering without assumed equal spacing.

Then add symbolic execution, arrays, probability, or general algebraic structures as actual use cases require them.

## Classes, objects, and composition

| Term | Meaning in this design |
|---|---|
| **Module** | A Python file grouping related definitions |
| **Class** | Defines the state and behavior of a kind of object |
| **Object / instance** | A particular instance of a class |
| **Attribute** | Information attached to an object |
| **Method** | Behavior accessed through an object |
| **Composition** | An object holds references to other objects |
| **Inheritance** | A class specializes another class |

Illustrative API shape—not a complete implementation:

```python
integers = Domain(name="integers", contains=integer_membership)

a = Value(data=3, domain=integers)
b = Value(data=5, domain=integers)

is_positive(a)       # Property
less_than(a, b)      # Relation
add(a, b)           # Operation returning a result object
```

```text
a ──has data────→ 3
  └─has domain─→ integers
                     ↑
b ──has data────→ 5   │
  └─has domain───────┘
```

Both values reference the same domain object. This is composition: a value **has a domain**. You do not need a separate subclass for every domain or every combination of domain, unit, and scale.

Use classes initially for `Domain` and your first value type. Use functions for most properties, relations, and operations. Introduce rule classes when you need to inspect, compose, register, serialize, or explain rules as data.

## Keep these mathematical roles distinct

| Role | Meaning |
|---|---|
| **Element** | An object considered as a member of a domain; not necessarily a dedicated base class |
| **Map / function** | Assigns one output to each input in its domain |
| **Operation** | A map used to combine or transform inputs |
| **Structure** | A domain equipped with specified operations, relations, and laws |
| **Symbol** | A mathematical placeholder |
| **Expression** | A mathematical construction, possibly containing symbols |
| **Statement** | A claim or condition |
| **Variable specification** | Describes an attribute and its allowed values and interpretation |
| **Dataset** | Observations of variables |
| **Distribution** | A probability model over possible values |

A function can itself be an element of a function space. These roles overlap; they do not form one simple hierarchy. An expression is already a mathematical object, even before evaluation.

## Computation and checking boundaries

- Keep the mathematical domain separate from the storage type: floating-point storage approximates real-number computation.
- Keep exact and approximate conversions explicit. A supplied float does not establish the user's intended exact rational value.
- For bulk data, wrap whole arrays rather than every scalar individually.
- Delegate existing algorithms instead of recreating them. A wrapper adds meaning and checks; speed improvements require measurement.
- Preserve an **unknown** result when a symbolic condition cannot be established.
- Distinguish declared assumptions, numerical checks, and proofs. Passing examples does not prove a general law.
- Shared specifications should be immutable where practical, or changes must trigger appropriate revalidation.
- Measurement scale informs valid interpretations; it does not alone select a statistical method.

## Later package growth

When responsibilities become substantial, split out:

- `measurement/`: quantities, scales, units, variable specifications.
- `data/`: observations, dimensions, missing values, array behavior.
- `symbolic/`: expression handling and symbolic integration.
- `probability/`: distributions and probability models.
- `backends/`: adapters for numerical and symbolic implementations.
- `structures/`: reusable algebraic structures and laws.

Keep domain and operation definitions independent of concrete backend implementations. A backend implements a calculation; the semantic layer determines input requirements and output meaning.

## First milestone

Build one domain, one value type, two properties, one relation, and one operation. Check valid and invalid membership, result domains, and errors. Then work through the length example before generalizing further.

## References

- [OpenStax: Levels of Measurement](https://openstax.org/books/introductory-business-statistics-2e/pages/1-3-levels-of-measurement) — introductory measurement scales.
- [Luce and Suppes: Representational Measurement Theory](https://suppescorpus.stanford.edu/sites/g/files/sbiybj32751/files/media/file/representational_measurement_theory_382.pdf) — empirical relations, numerical representations, and permissible transformations.
- [SageMath: Implementing Algebraic Structures](https://doc.sagemath.org/html/en/thematic_tutorials/coercion_and_categories.html) — parents, elements, categories, and conversion.
- [SymPy: Assumptions](https://docs.sympy.org/latest/guides/assumptions.html) — symbolic properties and unknown results.
- [SymPy: Numerical Computation](https://docs.sympy.org/latest/modules/numeric-computation.html) — moving symbolic expressions into numerical execution.
- [Pint: Temperature Conversion](https://pint.readthedocs.io/en/stable/user/nonmult.html) — offset units and temperature differences.
- [NumPy: Array Function Protocol](https://numpy.org/neps/nep-0018-array-function-protocol.html) — custom array-container dispatch; an advanced integration step.
