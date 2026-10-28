# Numbers, Data & Structure — Reference Sheet

## 1. The organizing framework

Use these **four questions** to understand a mathematical object or a dataset:

| Question | What to identify | Example: measuring sticks |
|---|---|---|
| **Domain** | What objects or values are involved? | Sticks; nonnegative real-valued lengths |
| **Structure** | What properties, relations, operations, and laws apply? | Longer than; equal length; joining lengths |
| **Interpretation** | What do the mathematical values represent? | Length in meters |
| **Invariance** | What remains meaningful when representation changes? | “Twice as long” survives conversion to feet |

This is a practical organizing framework. **Mathematical structures** formalize the ingredients; **measurement theory** studies how mathematical representations capture empirical relationships.

## 2. The ingredients and how they interact

| Ingredient | Meaning | Example |
|---|---|---|
| **Object** | A thing being considered | A person, stick, number |
| **Set / domain** | A collection of objects | All participants; the integers |
| **Property** | Something true or false of an object | “\(n\) is even” |
| **Attribute / variable** | Assigns a value to each object | A person’s height |
| **Relation** | Connects or compares objects | “A is taller than B”; \(a<b\) |
| **Function** | Assigns exactly one output to each input | Person \(\mapsto\) recorded height |
| **Operation** | A function used to combine or transform inputs | \(a+b\); \(-a\) |
| **Axiom / law** | A required condition on the structure | \(a+(b+c)=(a+b)+c\) |

**Connection:** a variable is modeled as a function. A property can be expressed as a condition on its value. Relations compare objects or values. Operations produce outputs; laws constrain their behavior.

Example: height \(h\) assigns values to people. The property “taller than 2 meters” is \(h(A)>2\). The relation “A is taller than B” is \(h(A)>h(B)\).

## 3. Number sets — what values are available?

\[
\mathbb N\subset\mathbb Z\subset\mathbb Q\subset\mathbb R\subset\mathbb C
\]

| Set | Meaning | Examples |
|---|---|---|
| **Natural** \(\mathbb N\) | Counting numbers; convention may include \(0\) | \(1,2,3,\ldots\) |
| **Integer** \(\mathbb Z\) | Whole numbers, negatives, and zero | \(-3,0,7\) |
| **Rational** \(\mathbb Q\) | \(p/q\), where \(p,q\in\mathbb Z,\ q\ne0\) | \(2/3,-4,0.125\) |
| **Real** \(\mathbb R\) | All points on the number line | Rationals, \(\sqrt2,\pi\) |
| **Complex** \(\mathbb C\) | \(a+bi\), with \(a,b\in\mathbb R,\ i^2=-1\) | \(3+2i,i,5\) |

- **Irrational:** real but not rational; decimal never terminates or eventually repeats.
- **Algebraic:** solves a nonzero polynomial with integer coefficients; \(1/2,\sqrt2,i\).
- **Transcendental:** not algebraic; \(\pi,e\).

**The set alone doesn’t specify every relevant structure:** \((\mathbb Z,+)\) identifies both the domain and the operation under study.

## 4. Measurement scales — which relationships do values represent?

| Scale | Meaningful information | Examples | Allowed change of representation |
|---|---|---|---|
| **Nominal** | Same/different category | Blood type, ID labels | Any distinct relabeling |
| **Ordinal** | Above + order | Rankings; low/medium/high | Any strictly increasing recoding |
| **Interval** | Above + equal differences | °C, °F | \(x'=ax+b,\ a>0\) |
| **Ratio** | Above + meaningful zero and ratios | Length, mass, duration | \(x'=ax,\ a>0\) |

**Invariance examples**

- **Ordinal:** \(1,2,3\rightarrow1,10,100\) preserves ranking, but changes gaps.
- **Interval:** °C → °F preserves ratios of differences, but changes ratios of readings.
- **Ratio:** meters → feet preserves “A is twice as long as B.”

**Meaningful does not mean numerically unchanged:** a length difference changes its numerical value when units change, while representing the same physical difference.

See [OpenStax: Levels of Measurement](https://openstax.org/books/introductory-business-statistics-2e/pages/1-3-levels-of-measurement) for an introduction, and [Luce & Suppes: Representational Measurement Theory](https://suppescorpus.stanford.edu/sites/g/files/sbiybj32751/files/media/file/representational_measurement_theory_382.pdf) for the formal connection between empirical relations, numerical representations, and permissible transformations.

## 5. Other classifications — separate questions

| Axis | Question | Examples |
|---|---|---|
| **Categorical / quantitative** | Category or numerical amount? | Blood type / height |
| **Discrete / continuous** | Separate possible values or a continuum? | Count / duration |
| **Scalar / vector** | One numerical component or multiple? | Temperature / velocity |
| **Deterministic / random** | Fixed by the model or modeled with uncertainty? | Fixed constant / die outcome |

These classifications combine:

| Variable | Combined description |
|---|---|
| Jersey number | Integer-coded, categorical, nominal |
| Finishing place | Integer-coded, ordinal; unequal gaps possible |
| Number of children | Integer-valued, quantitative, discrete, ratio |
| Temperature in °C | Real-valued model, quantitative, continuous, interval |
| Length | Real-valued model, quantitative, continuous, ratio |

**Recording precision is separate:** a continuous quantity can be recorded in rounded increments.

## 6. Common mathematical structures

| Structure | What it adds | Example |
|---|---|---|
| **Ordered set** | An order relation | \((\mathbb R,\le)\) |
| **Group** | An associative operation, identity, and inverses | Integers under addition |
| **Field** | Arithmetic including division by nonzero elements | \(\mathbb Q,\mathbb R,\mathbb C\) |
| **Vector space** | Vector addition and scalar multiplication obeying specified laws | \(\mathbb R^3\) |
| **Affine space** | Points and displacements, with no preferred origin | Positions; dates and durations as an analogy |
| **Metric space** | A distance function obeying specified laws | Euclidean plane |
| **Probability space** | Outcomes, measurable events, and probabilities | A model of a die roll |

See [Sharifi’s UCLA Abstract Algebra notes](https://math.ucla.edu/~sharifi/notes/algebra.html) for groups and fields, and [the vector-space section](https://www.math.ucla.edu/~sharifi/notes/algebra-ch05.html) for vector spaces.

## 7. Practical checks

1. Identify the **objects and variable**, not just the recorded numbers.
2. Separate the **number set** from the **measurement scale**.
3. Ask which **relations and operations** have an interpretation.
4. Check what survives a valid **relabeling or unit conversion**.
5. Choose statistical methods using the **question, study design, and model assumptions**, alongside the measurement scale.

**Numerical encoding does not create numerical meaning.**

## Python, these pieces map pretty naturally

| Mathematical concept | Code representation |
|---|---|
| Object | An instance of a class |
| Finite set | A `set` |
| Infinite domain | A type or membership predicate |
| Property | A function returning `bool` |
| Attribute / variable | A field or function extracting a value |
| Relation | A function comparing objects |
| Operation | A function producing an output |
| Axiom / law | A condition checked with tests or enforced by design |
| Measurement scale | Metadata describing meaningful comparisons |
| Representation change | A conversion function |
| Invariant | A condition checked before and after conversion |

## other stuff

| Tool | What it could contribute |
|---|---|
| **NumPy / SciPy** | Numerical storage, array operations, and numerical algorithms |
| **SymPy** | Symbolic expressions, exact mathematics, assumptions, and symbolic-to-numerical conversion |
| **[Pint](https://pint.readthedocs.io/en/stable/user/nonmult.html)** | Quantities, units, and distinctions such as temperature readings versus temperature differences |
| **[xarray](https://docs.xarray.dev/en/stable/user-guide/data-structures.html)** | Arrays with named dimensions, coordinates, and metadata |
| **[Pandera](https://pandera.readthedocs.io/en/latest/dataframe_schemas.html)** | Data schemas and validation rules |
| **[SageMath](https://doc.sagemath.org/html/en/thematic_tutorials/coercion_and_categories.html)** | Mathematical structures, their elements, and rules for operations and conversions |

| Part | Responsibility |
|---|---|
| **Domain** | Define membership: integers, positive reals, categories, vectors of a specified dimension |
| **Meaning** | Describe units, measurement scale, category ordering, and what a variable represents |
| **Representation** | Choose exact symbolic values, numerical arrays, precision, shape, and missing-value handling |
| **Operation rules** | Check inputs, determine output meaning, and select an implementation |
| **Evidence** | Record whether a condition is declared, checked, symbolically established, or unknown |

| Input and operation | Derived result |
|---|---|
| Length ÷ duration | Speed, with corresponding units |
| Temperature reading − temperature reading | Temperature difference |
| Temperature reading + temperature difference | Temperature reading |
| Count data → arithmetic mean | A potentially non-integer result |
| Ordinal categories → median | An ordered-category result, with an explicit convention |
| Nominal labels → arithmetic mean | An explanation that the encoding alone doesn’t justify this operation |
