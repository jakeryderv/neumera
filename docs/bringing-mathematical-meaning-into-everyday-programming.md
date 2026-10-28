# Bringing mathematical meaning into everyday programming

The idea is to combine Lean’s mathematical definitions and reasoning, Rust’s safety and native tooling, and Python’s accessibility through a coherent integration toolkit.

The preferred architecture is a Rust toolkit that integrates Lean and exposes mathematical objects, executable operations, and reasoning through natural Rust APIs. A separate Python adapter would use PyO3 and maturin to make those capabilities accessible from Python.

The goal is to preserve mathematical meaning across these interfaces so it remains useful while constructing programs: checking requirements, deriving properties, composing guarantees, and explaining unresolved obligations.

**1. Export computation and mathematical meaning together**

Ordinary bindings expose functions, values, and data structures. This toolkit would additionally preserve:

- Domains and representations.
- Input requirements.
- Relationships between inputs and outputs.
- Established properties and applicable theorems.
- Connections to the definitions and evidence supporting those properties.

An exported `double` operation would provide more than an integer-to-integer function. Its interface would preserve the relationship between its input and output.

Programmers should not need to recreate that relationship manually in every language.

**2. Declare what is needed; derive what follows**

The programmer supplies definitions and assumptions. The system makes their established consequences available automatically.

Declaring a symbolic value as real permits reasoning from the properties of real numbers. Rationality remains unspecified until additional evidence establishes it.

Evidence can come from a declaration, construction, operation, checked condition, or proof.

If two values are constructed as rationals, their sum should be recognized as rational without another annotation. For an exact real `x`, the expression `x - x` can be established as zero and therefore rational even when the rationality of `x` is unknown.

The guiding principle is to require the weakest sufficient assumptions and avoid asking programmers to restate derivable facts.

**3. Knowledge follows values and expressions**

The interface should distinguish a value’s domain, computational representation, and established properties.

For example:

- A value is real.
- Its concrete representation is an exact rational.
- It is positive.
- Another expression equals twice that value.
- A construction or proof establishes these facts.

Additional knowledge can accumulate without repeatedly changing the value’s representation.

Symbolic expressions should retain their mathematical structure. Concrete computations should preserve or expose relevant guarantees where supported.

Conversions to ordinary Rust or Python values must make clear which mathematical information remains available. Converting an object to a plain integer should not imply that subsequent arbitrary host-language operations are formally tracked.

**4. Guarantees compose through operations**

An operation’s established properties become available to subsequent operations.

If concatenation combines vectors of lengths `m` and `n`, its result has length `m + n`. That relationship can satisfy another function’s requirements without repeated checks or declarations.

The same principle applies to ordering, bounds, uniqueness, compatible dimensions, conservation properties, and valid state transitions.

The interface should support concrete computation and symbolic construction, with explicit connections to the corresponding Lean definitions.

**5. Rust is the primary integration layer**

The core toolkit would be distributed as Rust crates that manage the Lean connection and provide usable Rust interfaces.

Responsibilities would include:

- Lean runtime initialization and object ownership.
- Supported conversions between Rust and Lean representations.
- Generated interfaces to Lean definitions.
- Access to reasoning and proof-checking services.
- Preservation of mathematical relationships and evidence.
- Errors, diagnostics, and resource management.

“Native to Rust” means that developers can use these capabilities through natural Rust APIs and established Cargo workflows.

Lean remains responsible for its mathematical judgments. Depending on the crate does not automatically make Rust’s compiler understand arbitrary Lean dependent types or verify surrounding Rust code.

**6. Cargo coordinates compilation across languages**

Rust compiles the Rust components. Lean and Lake compile the Lean components.

Build helpers would coordinate these steps through Cargo, aiming to provide one ordinary build workflow:

1. Locate the supported Lean toolchain and dependencies.
2. Check and build the Lean components.
3. Generate the supported Rust interfaces and mathematical descriptions.
4. Compile Rust and link the required components.
5. Collect artifacts needed for execution and distribution.

The toolkit should centralize compatibility and build details so downstream developers do not repeatedly implement them.

Lean metaprogramming could inspect supported declarations and generate interfaces from their definitions and types. Explicit mapping rules would determine which features become ordinary Rust values and which require dedicated wrappers or symbolic objects.

**7. Python is a separate adapter over the Rust API**

A Python package could consume the Rust toolkit through PyO3 and use maturin for building and packaging.

The Python layer would focus on Python-facing classes, functions, exceptions, and usability. It should reuse the Rust integration rather than implement another Lean bridge.

The intended structure is:

```text
Lean definitions, implementations, and proofs
                       ↓
Rust integration and mathematical interface
                       ↓
          Rust applications and libraries
                       ↓
              Optional PyO3 adapter
                       ↓
          Python package built with maturin
```

The final package must still contain or locate the Lean libraries and resources it requires. The preferred experience for prebuilt Python wheels is installation without a separate Rust or Lean toolchain. Building from source may require both.

Packaging support must account for native libraries, mathematical artifacts, platform compatibility, and any resources required for runtime reasoning.

**8. Integration deepens incrementally**

The toolkit can provide several levels of support:

| Level | Capability |
|---|---|
| Executable bindings | Call supported Lean implementations |
| Mathematical objects | Preserve domains, expressions, relationships, and evidence |
| Reasoning-aware operations | Derive consequences and establish requirements automatically |
| Source and editor integration | Check supported surrounding code during development |

The first levels can work through library APIs and generated wrappers. Broader checking of Rust or Python source requires additional tooling with clearly defined language support.

The interface should make the extent of its reasoning visible.

**9. Execution and reasoning remain connected**

An implementation and its guarantees must agree. Changes should trigger checks of affected guarantees and regeneration of relevant interface information.

The system should avoid a disconnected executable implementation and mathematical model that can silently drift apart.

Optimized Rust implementations could participate, but their relationship to the mathematical interface must be established. Compilers, runtimes, conversions, and wrappers remain execution assumptions unless separately verified.

Representations must be mathematically honest. Exact integers, fixed-width integers, exact reals, symbolic expressions, and floating-point approximations have different behavior. Guarantees must account for the representation actually used.

**10. Reasoning is automatic where possible**

The system retains definitions, known facts, assumptions, and reusable results. It derives relevant consequences when an operation needs them.

Routine obligations should resolve automatically. Difficult cases may require a stronger assumption, supporting lemma, runtime check, or explicit proof.

Feedback should distinguish an established requirement, a demonstrated contradiction, an unresolved obligation, and an explicitly trusted assumption.

Failure to find a proof does not establish that a claim is false.

Runtime checks can establish executable properties of external inputs. Properties without a general executable test require suitable constructions or evidence.

Mutation must preserve established properties, establish replacements, or invalidate knowledge that no longer holds.

**11. A focused first demonstration**

Existing Lean–Rust integration projects should be evaluated before deciding which infrastructure to build.

The first prototype should expose a small collection of Lean-defined operations over exact integers, rationals, and immutable vectors through Rust.

A Rust developer should be able to:

- Construct concrete and symbolic mathematical objects.
- Compose operations through ordinary APIs.
- Reuse inferred lengths and arithmetic relationships.
- Satisfy requirements using existing evidence.
- Receive clear feedback when additional information is needed.

A small Python adapter should then expose the same capabilities through PyO3 and maturin.

Calling a compiled function establishes the integration foundation. Composing its mathematical guarantees across operations demonstrates the distinctive idea.

The broader goal is a reusable Rust foundation that brings Lean’s mathematical meaning into practical programming and makes those capabilities available to Python through established tooling.
