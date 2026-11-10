# ADR 0003 — Build Orchestration

## Context

The project mixes a C++ host and Rust static libraries. MSVC is available via VS 2022 Build Tools but is not on the default PATH. CMake may be the VS-bundled binary or a winget Kitware install.

## Problem

Who owns the build graph without duplicating dependency management?

## Alternatives

1. **Cargo-primary** with `cc`/`cmake` crates building C++ — awkward for a C++-owned Vulkan host and IDE presets.
2. **CMake-primary** invoking Cargo for Rust staticlibs — clear host ownership.
3. **Two independent builds** manually linked — fragile and undocumented.

## Choice

**CMake is the native host build.** It builds `runtime-bridge` via Cargo (`--release`/`--debug` matching the CMake config) and links the resulting `runtime_bridge.lib` into `vertical_shift`.

Cargo exclusively owns the Rust workspace (`crates/*`).

Happy path preset: `windows-msvc-debug`.

Developer shell: `VsDevCmd.bat -arch=x64` before CMake when `cl` is not on PATH.

## Consequences

- Document exact commands in the README.
- `CMakePresets.json` must locate Vulkan SDK via `VULKAN_SDK`.
- CI (future) must replicate VsDevCmd + Cargo + CMake.

## Validation

- `cmake --preset windows-msvc-debug` configures and builds the executable.
- Changing only Rust sources rebuilds the staticlib through the CMake custom target.
