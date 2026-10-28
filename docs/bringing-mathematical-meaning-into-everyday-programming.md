# Bringing mathematical meaning into everyday programming

The idea is to combine Lean's mathematical definitions and reasoning, Rust's safety and native tooling, and Python's accessibility through a coherent integration toolkit.

The preferred architecture is a Rust toolkit that integrates Lean and exposes mathematical objects, executable operations, and reasoning through natural Rust APIs. A separate Python adapter would use PyO3 and maturin to make those capabilities accessible from Python.

The goal is to preserve mathematical meaning across these interfaces so it remains useful while constructing programs: checking requirements, deriving properties, composing guarantees, and explaining unresolved obligations.

This document is organized in four tiers so that a reader can tell what is being built now, what already exists to build on, what is proposed, and what is deferred:

1. [What the first prototype builds](#what-the-first-prototype-builds)
2. [The idea](#part-1-the-idea)
3. [Established infrastructure](#part-2-established-infrastructure)
4. [Proposed mechanisms](#part-3-proposed-mechanisms)
5. [Longer-term research](#part-4-longer-term-research)
6. [First demonstration](#part-5-first-demonstration)

## What the first prototype builds

The prototype chooses one narrow, explicit combination of the mechanisms described in Part 3.

**In scope**

- A small Lean library over exact integers, exact rationals, and immutable vectors, with theorems about a handful of operations (concatenation, sorting, merging, searching).
- Build-time verification: definitions, theorems, and the exported interface are checked by Lean once, during the build.
- A Rust crate that calls the compiled Lean implementations through the C ABI and wraps results in objects that carry established properties.
- Runtime propagation of properties through supported operations, implemented as hand-written Rust rules that transcribe specific Lean theorems. Every quantity tracked at runtime is concrete.
- A contract manifest, produced by the Lean build and consumed by the Rust build, so that every guarantee-bearing Rust rule must reference a Lean contract that was checked in the same build. Removing or weakening a required contract fails the build.
- A Python adapter over the Rust crate that exposes the same objects and operations as library classes.

**Out of scope for v0**

- Runtime Lean reasoning. No Lean elaborator, kernel, or REPL runs in the shipped artifact. The compiled Lean implementations and the Lean runtime they depend on do ship; see Part 3.
- Symbolic quantities at runtime. Symbolic values are specification objects and are reasoned about in Lean at build time only. This is a simplification for the prototype, not a claim that symbolic support is unmanageable.
- Verification of independently optimized Rust implementations against Lean definitions.
- Floating-point numerical correctness.
- Static checking of surrounding Rust or Python source.

**Demonstration**

Concatenation serves as the warm-up for the foreign function interface itself. The distinctive demonstration is sortedness, because it is a property that is expensive to re-check by inspection and so shows where propagation earns its keep:

- Sorting external input establishes sortedness through a runtime construction.
- Merging two sorted vectors yields a sorted vector without an O(n) re-check, by a transcribed Lean theorem.
- Binary search requires a sorted input and is satisfied by provenance rather than by scanning.

The contracts must be strong enough to rule out trivial implementations, and the Rust wrapper must make the property unforgeable. Part 5 spells out both.

**What v0 does and does not validate**

A working demonstration validates the integration (Lean compiled and called from Rust and Python, one build workflow) and the preservation of previously established guarantees through concrete operations. It does not validate the broader experience described in Part 1, where mathematical consequences become available dynamically as a program is written. That experience depends on symbolic quantities and on-demand reasoning, both deferred. The working hypothesis is that the knowledge language and provenance model built for v0 are the foundation that later inference needs, rather than a detour from it. The demonstration cannot settle that hypothesis; it can only show whether the foundation is usable.

## Part 1: The idea

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

If two values are constructed as rationals, their sum should be recognized as rational without another annotation. For a symbolic real `x`, the expression `x - x` can be established as zero and therefore rational even when the rationality of `x` is unknown. This is a specification-level example: it is reasoned about in Lean, not evaluated at runtime (see Part 2 on real numbers).

The guiding principle is to require the weakest sufficient assumptions and avoid asking programmers to restate derivable facts.

**3. Knowledge follows values and expressions**

The interface should distinguish a value's domain, computational representation, and established properties.

For example:

- A value is real.
- Its concrete representation is an exact rational.
- It is positive.
- Another expression equals twice that value.
- A construction or proof establishes these facts.

Additional knowledge can accumulate without repeatedly changing the value's representation.

Symbolic expressions should retain their mathematical structure. Concrete computations should preserve or expose relevant guarantees where supported.

Conversions to ordinary Rust or Python values must make clear which mathematical information remains available. Converting an object to a plain integer should not imply that subsequent arbitrary host-language operations are formally tracked.

**4. Guarantees compose through operations**

An operation's established properties become available to subsequent operations.

If concatenation combines vectors of lengths `m` and `n`, its result has length `m + n`. That relationship can satisfy another function's requirements without repeated checks or declarations.

A relationship retained by a wrapper is different from a relationship enforced by the Rust compiler. The toolkit retains relationships in wrapper objects. It does not promise that stable Rust can express arbitrary length arithmetic in const-generic return types (see Part 2).

The same principle applies to ordering, bounds, uniqueness, compatible dimensions, conservation properties, and valid state transitions.

The interface should support concrete computation and symbolic construction, with explicit connections to the corresponding Lean definitions.

**5. Reasoning is automatic where possible**

The system retains definitions, known facts, assumptions, and reusable results. It derives relevant consequences when an operation needs them.

Routine obligations should resolve automatically. Difficult cases may require a stronger assumption, supporting lemma, runtime check, or explicit proof.

Feedback should distinguish an established requirement, a demonstrated contradiction, an unresolved obligation, and an explicitly trusted assumption.

Failure to find a proof does not establish that a claim is false.

Runtime checks can establish executable properties of external inputs. Properties without a general executable test require suitable constructions or evidence.

Mutation must preserve established properties, establish replacements, or invalidate knowledge that no longer holds.

**6. Lean's contribution is specific**

Lean contributes checked derivations and reusable formal definitions and theorems. It is not required merely to attach descriptive metadata to values; a Rust or Python library can do that alone. The toolkit should justify each use of Lean by one of those two contributions.

## Part 2: Established infrastructure

Facts about the surrounding ecosystem that the design relies on or must work around. These should be re-verified before building on them.

**Lean compilation and export.** Lean 4 compiles to C and ships a C runtime. Declarations marked with `@[export]` receive stable C symbols. Lake builds Lean projects and their dependencies. Proofs are computationally irrelevant and are erased from compiled code; a theorem has no runtime representation.

**Rust bindings to the Lean runtime.** The `lean-rs-sys` crate provides raw bindings to the Lean 4 C ABI, including runtime initialization and toolchain pinning via a checksum of `lean.h`. Its companion `lean-rs` crate is described as the safe front door. Both should be evaluated before writing a new binding layer; runtime initialization and object ownership may already be covered.

**Python routes to Lean.** Existing Python integrations (LeanInteract, PyPantograph, LeanDojo) drive a Lean process through a REPL or language-server-style protocol. They target proof interaction and data extraction, not shipping compiled Lean code inside a Python package. They are the reference point for the deferred runtime-reasoning mechanism, not for the prototype.

**Rust to Lean verification.** Aeneas extracts Rust programs into Lean for verification. It works in the opposite direction from this toolkit and is substantial research infrastructure. It defines the scope of what "verifying an optimized Rust implementation" would actually require.

**Real numbers in Mathlib.** Mathlib defines the reals as a quotient of Cauchy sequences. Order, inverse, and equality decisions are marked noncomputable. Mathlib's real-number operations are not ordinary executable numerical operations, and this document must not imply otherwise.

**Floating point in Lean.** The Lean reference now gives `Float` a logical model over bit patterns and specifies arithmetic (add, sub, mul, div, sqrt), comparisons, and conversions against it. Transcendental functions remain opaque, the model is documented as not intended to support general lemmas, and there is no library for numerical error reasoning. Mathlib's theorems are about the mathematical reals. The gap between what can be proved about and what is computed with is still widest for floating-point numerics, and the state of the specification should be checked against the Lean version pinned for the prototype.

**Const generics in Rust.** Arithmetic on const generic parameters in types (such as a return type of length `M + N`) is unstable as of this writing. Length relationships must be retained by wrappers rather than enforced by the type checker.

## Part 3: Proposed mechanisms

**1. Three mechanisms, chosen separately**

Guarantees can live in three different places. The mechanisms can coexist, but each has a distinct trust model, cost, and shipping footprint, and a design must say which it is using for each guarantee.

| Mechanism | What happens | When | Trust and cost |
|---|---|---|---|
| Build-time verification | Lean checks definitions, general theorems, and the exported interface | Once, during the build | Lean kernel; no proof-checking machinery ships |
| Runtime propagation | Rust applies established rules to carry properties through supported constructions and operations | On each operation | The rule-application machinery; see below |
| Runtime Lean reasoning | A Lean process resolves obligations the existing rules cannot | On demand | Lean elaboration latency plus shipping the elaborator and its libraries |

The first prototype uses the first two and excludes the third.

In every case the Lean compiler, the Lean runtime, the foreign function interface, conversions, wrappers, and the association between tracked properties and actual values remain execution assumptions unless separately verified. No mechanism below removes them from the trusted base.

**2. Runtime propagation and its trusted base**

Runtime propagation means the Rust crate holds a small knowledge language (concrete lengths, orderedness, bounds, provenance) and a rule-application engine that interprets it. Each rule instantiates a theorem that Lean proved once. The engine and its connection to actual values are the trusted base, and there are three ways to treat it:

| Option | Trusted base | Cost |
|---|---|---|
| Transcribed | Hand-written Rust transcription of each theorem | Cheapest; drift between Lean and Rust is the risk; N rules to audit |
| Generated | Rust rule code emitted from the theorem's statement by Lean metaprogramming | Reduces manual duplication; the generator and the theorem-to-operation mapping become the audit surface instead of N transcriptions. Generated code is not thereby verified; the generator can still emit wrong Rust or attach a theorem to the wrong operation |
| Verified generation | As above, with the generator proven correct or its output independently checked | Research; not planned |
| Evidence-checked | Rust records a derivation trace; a checker replays it at a boundary | Replay is theorem instantiation, not proof search, which suggests it could be cheaper than full Lean; footprint and latency are unmeasured. Needs a checker, the project's library and dependencies loaded, and a reliable association between the trace and the actual values |

The prototype uses transcription with tests. Generation is the stated next step. Evidence-checked replay is a proposed path toward runtime reasoning without proof search; whether it is practical must be measured, and external kernel checkers are candidates for the checker role.

Transcription on its own leaves the theorem-to-rule association implicit, which means a theorem can be deleted or weakened without any Rust code noticing. The prototype therefore adds a contract manifest (see section 8 below and the design example). The manifest checks that each rule references a contract with the expected statement from the same build. It does not check that the rule applies the contract correctly; the rule bodies remain trusted under every option in the table.

**3. Runtime quantities are concrete**

The moment a requirement `length = k` must be satisfied by a symbolic `m + n` when `k` was built as `n + m`, the engine needs to normalize arithmetic terms. A bounded normalizer with a fixed rule set that answers "unresolved" outside it is a legitimate design and matches Part 1's treatment of unresolved obligations. But each normalization rule is one more transcribed theorem, so a symbolic layer multiplies the trust question above before the first version of it has been exercised. The prototype therefore keeps every tracked quantity concrete at runtime. Symbolic quantities are specification objects and are reasoned about in Lean at build time. The same rule covers symbolic reals, which never reach the runtime as executable values. This is a scope choice for v0, not a boundary of the approach.

**4. When propagation pays off**

With concrete values, a property that is cheap to inspect gains nothing from being carried. The length of a concatenation is O(1) to read, so the value is its own evidence. Propagation earns its keep when the property is expensive or impossible to check by inspection: sortedness, uniqueness, being a permutation of another vector, positive definiteness. The demonstration in Part 5 is chosen on that basis.

**5. Rust is the primary integration layer**

The core toolkit would be distributed as Rust crates that manage the Lean connection and provide usable Rust interfaces.

Responsibilities would include:

- Lean runtime initialization and object ownership (possibly delegated to existing crates).
- Supported conversions between Rust and Lean representations.
- Generated interfaces to Lean definitions.
- Preservation of mathematical relationships and evidence.
- Errors, diagnostics, and resource management.

"Native to Rust" means that developers can use these capabilities through natural Rust APIs and established Cargo workflows.

Lean remains responsible for its mathematical judgments. Depending on the crate does not automatically make Rust's compiler understand arbitrary Lean dependent types or verify surrounding Rust code.

**6. Cargo coordinates compilation across languages**

Rust compiles the Rust components. Lean and Lake compile the Lean components.

Build helpers would coordinate these steps through Cargo, aiming to provide one ordinary build workflow:

1. Locate the supported Lean toolchain and dependencies.
2. Check and build the Lean components.
3. Generate the supported Rust interfaces and mathematical descriptions.
4. Compile Rust and link the required components.
5. Collect artifacts needed for execution and distribution.

The toolkit should centralize compatibility and build details so downstream developers do not repeatedly implement them.

Lean metaprogramming could inspect supported declarations and generate interfaces from their definitions and types. Explicit mapping rules would determine which features become ordinary Rust values and which require dedicated wrappers. The same machinery is the route to the "verified" option for rule code.

**7. Python is a separate adapter over the Rust API**

A Python package could consume the Rust toolkit through PyO3 and use maturin for building and packaging.

The Python layer would focus on Python-facing classes, functions, exceptions, and usability. It should reuse the Rust integration rather than implement another Lean bridge.

Python preserves mathematical information through library objects and operations. This prototype does not perform static checking of Python callers; requirements are discharged at runtime from provenance or by explicit checks. Generated type stubs still let an ordinary Python type checker distinguish the wrapper classes, which is a convenience rather than a guarantee the toolkit relies on.

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

Because the prototype excludes runtime Lean reasoning, no Lean development toolchain or runtime proof-checking environment is required to use the package. The distribution includes the compiled Lean implementations and their required runtime dependencies: compiled Lean code uses runtime services for allocation, reference counting, task scheduling, and primitive operations on built-in types, and those services ship with the package, linked statically or as a bundled library. The preferred experience for prebuilt Python wheels is installation without a separate Rust or Lean toolchain. Building from source may require both. Wheel size is one of the measurements the demonstration should report.

**8. Execution and reasoning remain connected**

An implementation and its guarantees must agree. Changes should trigger checks of affected guarantees and regeneration of relevant interface information.

The system should avoid a disconnected executable implementation and mathematical model that can silently drift apart. In the prototype, the drift risk is concentrated in the hand-written rule transcriptions, which is why generation is the next step.

A passing Lean build is not sufficient evidence that the contracts Rust relies on still hold. Deleting a theorem, or weakening its statement, leaves the executable exports unchanged and the Rust wrappers compiling. The build must therefore check specific expected contract statements, not merely that the Lean project builds. The mechanism is a contract manifest:

- A Lean contracts module restates each required contract verbatim and discharges it by the named theorem, so deletion or weakening fails elaboration.
- The build audits each exported contract's transitive axiom dependencies and rejects `sorryAx` and any axiom outside an explicit allowlist, since a `sorry` elaborates with only a warning and leaves the compiled code unchanged.
- The same build emits a manifest associating each contract with its name, its statement, its axioms, and the executable export it describes. The manifest identifies the build by the toolchain version, the executable hash, the hashes of the checked Lean artifacts, and the pinned dependency revisions. The executable hash alone cannot distinguish proof-only changes, because proofs are erased from compiled code.
- The Rust build consumes the manifest, recomputes and compares the executable, checked-artifact, and dependency hashes and the toolchain version against the artifacts it is about to link, and only then generates one constant per contract. Every guarantee-bearing Rust rule references its contract's constant, so a missing contract is a compile error. Lean contracts with no Rust rule are permitted.
- Negative tests confirm that removing a required contract, weakening it, replacing its proof with `sorry`, or pairing a manifest with a different build each fail the build.

The manifest establishes that the rules reference real, checked contracts. Whether the rules apply those contracts correctly remains trusted code.

Representations must be mathematically honest. Exact integers, fixed-width integers, symbolic reals, and floating-point approximations have different behavior. Guarantees must account for the representation actually used.

**9. Integration deepens incrementally**

| Level | Capability | Status |
|---|---|---|
| Executable bindings | Call supported Lean implementations | Prototype |
| Mathematical objects | Preserve domains, representations, concrete properties, and evidence | Prototype |
| Propagation-aware operations | Carry established properties through supported operations | Prototype (trusted rules) |
| Reasoning-aware operations | Resolve obligations the rules cannot | Research |
| Source and editor integration | Check supported surrounding code during development | Research |

The interface should make the extent of its reasoning visible.

## Part 4: Longer-term research

These are kept as direction, not commitments. Each is a substantial project in its own right.

- **Runtime Lean reasoning.** Resolving obligations on demand by running Lean, either through evidence-checked replay or through proof search. Shipping cost and latency are the open problems.
- **Symbolic quantities at runtime.** Lifting the concrete-only cap requires a normalizer or solver for the knowledge language and a story for keeping it faithful to Lean.
- **Verification of optimized Rust implementations.** Establishing that a hand-optimized Rust implementation agrees with a Lean definition. Aeneas defines the shape of this problem.
- **Floating-point correctness.** Relating floating-point computations to statements about the reals. Lean currently offers little theory here.
- **Source and editor integration.** Checking surrounding Rust or Python code against the mathematical interface during development.

## Part 5: First demonstration

**Step zero: evaluate existing work.** Before building infrastructure, evaluate `lean-rs-sys` and `lean-rs` for the binding layer, and record what they cover and what they do not. Decide whether to depend on them, fork them, or replace them.

**Step one: one operation through all three languages, with the manifest.** Pin a Lean version. Define vector concatenation in Lean, prove its length contract, export it, build with Lake under Cargo, call it from Rust through the binding layer, and expose it from Python. Put the length contract through the full manifest mechanism: contracts module, emitted manifest tied to the build artifact, generated Rust constant, and a rule that references it. Run the negative test showing that removing or weakening the contract fails the build. The length contract is trivial and its saving is nil; the point of this step is that every piece of machinery exists and is tested before the first interesting contract is added.

**Step two: the distinctive demonstration.** Add sorted vectors.

- In Lean: define `sort`, `merge`, `isSorted`, and `binarySearch` over vectors of exact integers, all using one ordering relation. Prove that `sort` returns a sorted permutation of its input (sortedness alone admits an implementation that returns an empty vector), that `merge` of two sorted inputs returns a sorted permutation of their concatenation, that `isSorted` decides the sortedness predicate, and that `binarySearch` returns an index holding the target when it succeeds and that the target is absent when it fails. `isSorted` must be a linear adjacent-pair scan, not the generic decision procedure for pairwise predicates, which is quadratic; the contract connecting the scan to the predicate is part of the proof obligation. Core Lean already provides sortedness and permutation predicates and a verified merge sort, so much of this may be reuse; confirm against the pinned version.
- In Rust: wrap vectors in a type whose constructor is private and whose contents cannot be mutated through the public API, so that the sortedness property cannot be forged or outlived. Operations take their inputs by reference; ownership of the underlying Lean objects is handled inside the integration layer, so Rust and Python have the same semantics. Because object parameters of exported Lean functions are owned, the binding layer retains an additional reference before passing a borrowed Rust handle to Lean; the convention is confirmed against the pinned toolchain during implementation. Sorting external input establishes the property through a runtime construction. Checking external input with the exported `isSorted` establishes it through a runtime check, using the same relation Lean proved about. Merging two sorted inputs establishes it through a transcribed rule, without re-checking. Binary search requires the property and reports a clear error, naming the missing property and how to establish it, if it is absent. Explicitly forgetting the property is allowed; silently losing it is not. Each rule references its contract through the manifest.
- In Python: the same objects and operations, as library classes with the same encapsulation.

The companion document [Design example: sorted vectors](design-example-sorted-vectors.md) works this through in detail, including the exact contracts, the Rust surface, and the enumerated trusted steps.

A Rust or Python developer should then be able to:

- Construct concrete mathematical objects.
- Compose operations through ordinary APIs.
- Reuse established properties across operations.
- Satisfy requirements using existing evidence.
- Receive clear feedback when additional information is needed.

**What the demonstration must show explicitly.** For each guarantee, where it lives: checked by Lean at build time, propagated by a Rust rule at runtime, or established by a runtime check. The demonstration should make the trusted base visible, so that a reader can see exactly which Rust code must be correct for the guarantees to hold. Its performance measurement should compare `merge` alone against the same `merge` followed by an independent sortedness scan, which isolates the cost that preserving the guarantee saves.

Calling a compiled function establishes the integration foundation. Composing its mathematical guarantees across operations demonstrates the distinctive idea.

The broader goal is a reusable Rust foundation that brings Lean's mathematical meaning into practical programming and makes those capabilities available to Python through established tooling.
