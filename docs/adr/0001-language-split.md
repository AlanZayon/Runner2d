# ADR 0001 — Language Split

## Context

VERTICAL SHIFT needs a native specialized runtime with extreme performance goals, safe gameplay/procgen logic, and a thin interoperability surface. The development host is Windows x64 with MSVC and Rust available.

## Problem

How should C++, Rust, C, and Assembly be assigned so each language earns its place without decorative multi-language wrappers?

## Alternatives

1. **Rust-only** — simplest FFI story; weaker fit for deep Vulkan/Windows GPU control and existing MSVC ecosystem tooling.
2. **C++-only** — strong for renderer/platform; weaker default memory safety for complex procgen/validation state machines.
3. **Mixed with per-object FFI** — demonstrates languages but destroys performance.
4. **C++ host + Rust sim via C ABI batches** — specialized split with a narrow boundary.

## Choice

Adopt alternative 4:

| Language | Responsibility |
|----------|----------------|
| C++ | Host process, Win32 platform, game loop, Vulkan renderer, GPU resources |
| Rust | Gameplay rules, procgen, validation, difficulty director, headless sim, tools |
| C | Shared ABI header only (`vertical_shift_abi.h`) |
| Assembly | Deferred until measured bottlenecks with scalar fallbacks |

## Consequences

- Dual build systems (CMake + Cargo) must stay documented and thin.
- FFI must stay batched; no per-entity crossing.
- Assembly is explicitly out of scope until Phase 10 profiling.

## Validation

- Phase 1 executable links Rust `runtime-bridge` into the C++ host and logs the ABI version.
- ABI round-trip unit tests pass in Rust.
- Later benchmarks compare batch vs naïve FFI if regressions appear.
