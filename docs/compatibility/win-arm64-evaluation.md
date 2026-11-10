# Windows ARM64 — evaluation checklist (not implemented)

Phase 12 documents acceptance only. Do **not** mark Win-ARM64 supported until every item passes on real hardware (or a verified ARM64 VM with Vulkan).

## Prerequisites

- VS 2022 ARM64 / cross tools (`cl` targeting ARM64)
- Rust target `aarch64-pc-windows-msvc`
- Vulkan ICD on the ARM64 device
- Decision: native ARM64 build vs x64 emulation (emulation ≠ “supported ARM64”)

## Acceptance

1. CMake preset `windows-msvc-arm64-release` configures and links.
2. Window + Vulkan present; sprites visible.
3. Keyboard + mouse-gesture mapping match x64.
4. `--cpu-bench 1000 --seed 42` completes; numbers logged in baselines with CPU model.
5. Pack script variant or documented copy of `assets/` + `shaders/` next to exe.
6. Matrix row updated: Build/Run/Packaged = Yes with device named.

## Out of scope until then

- Shipping Store ARM64 package, WoA-specific input quirks as “done” without test.
