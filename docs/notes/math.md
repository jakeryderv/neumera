# neumera.math

```
src/neumera/math/
├── __init__.py
├── objects.py       # The things you work with
├── domains.py       # Which objects or values belong
├── properties.py    # Conditions on one object
├── relations.py     # Connections between objects
└── operations.py    # Calculations that produce results
```

## where each concept fits

| Module | Main question | Example |
|---|---|---|
| **Objects** | What thing am I representing? | A length quantity, vector, expression, dataset |
| **Domains** | What values are allowed? | Positive reals, integers, three-dimensional vectors |
| **Properties** | What is true of this object? | Positive, finite, nonempty |
| **Relations** | How do these objects connect or compare? | Equal, longer than, compatible units |
| **Operations** | What result can I produce from these inputs? | Add lengths, subtract readings, compute a mean |

#### Important distinction: "Property" has two common meanings:

| Meaning | Example | Starting location |
|---|---|---|
| **Descriptive attribute** | Value, domain, unit, shape, measurement scale | Fields on the object |
| **Mathematical condition** | Is positive, is finite, is symmetric | Predicate in `properties.py` |

Python's `@property` is a programming mechanism for attribute access. It does not itself represent a mathematical property

## Example to connect everything

Suppose `a` and `b` represent lengths:

```
OBJECTS
    a: 2 meters
    b: 30 centimeters

DOMAINS
    Their magnitudes belong to nonnegative real numbers.

ATTRIBUTES
    Each quantity has a magnitude, unit, and domain.

PROPERTIES
    Is a finite?
    Is b positive?

RELATIONS
    Are their units compatible?
    Is a longer than b?

OPERATIONS
    Add them:
        check compatibility
        convert units
        calculate
        return a length quantity: 2.3 meters
```

The operation uses properties and relations to check its inputs. It then creates a result with the appropriate attributes

## What needs a class?

Use classes for objects that carry state or specifications. Start with ordinary functions for many properties, relations, and operations:
