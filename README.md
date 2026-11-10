# VERTICAL SHIFT

Native specialized 2D infinite vertical runner runtime — built from scratch (no game engine).

**Phase:** 12 — spec compliance (`0.12.0-phase12`, ABI **1.7**).

## What this is

A portfolio-grade **native game runtime** plus the VERTICAL SHIFT runner: own game loop, entities, collisions, procedural segments with headless validation, streaming, abilities, Falling Mode, Vulkan 2D renderer, audio mixer, UI, and persistence — without Unity/Unreal/Godot/Bevy/etc.

## Built from scratch vs low-level libraries

| Own systems | Low-level libs used |
|-------------|---------------------|
| Game loop, ECS-ish storage, collisions, abilities, procgen, streaming, director, Falling Mode, UI layout, save format | **Vulkan** (via **volk** + headers), **miniaudio**, **stb_image** / **stb_truetype**, Win32 / Android NDK |

## Languages

| Language | Role |
|----------|------|
| **C++** | Host, platform, Vulkan renderer, audio, UI, packaging |
| **Rust** | Simulation core (`runtime-core`) — safe gameplay logic |
| **C** | Shared ABI header (`vertical_shift_abi.h`) |
| **Assembly** | **Not shipping** — measured in Phase 10; deferred (ADR 0012) |

## Portfolio — platforms

| Status | Target |
|--------|--------|
| **Verified** | Windows 10/11 x64 (MSVC) + Vulkan — Build / Run / Packaged |
| **Scaffold** | Android arm64-v8a (device checklist still required) |
| **Open / deferred** | Linux x64, iOS/macOS, Windows ARM64, 2nd graphics backend |

Details: [docs/compatibility/matrix.md](docs/compatibility/matrix.md) · [ADR 0014](docs/adr/0014-phase12-spec-compliance.md).

## Controls

| Input | Action |
|-------|--------|
| **Enter** / **Space** / **Tap** | Title / GameOver → start or restart |
| Auto-run | Forward (no strafe except Falling) |
| **A** / **D** | Strafe in Falling Mode |
| **Space** / swipe **up** | Jump |
| **Shift** / **J** / swipe **right** | Dash |
| **S** / swipe **down** | Roll |
| **K** / **E** / **Tap** | QTE confirm |
| **R** | Restart |
| **Esc** / **Back** | Pause / resume |
| **F3** | Debug overlay + AABB draw |
| **1–4** | Quality Low→Ultra |

## Build / run (Windows)

```bat
"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64
cmake --preset windows-msvc-debug
cmake --build --preset windows-msvc-debug
cmake --preset windows-msvc-release
cmake --build --preset windows-msvc-release
cmake --preset windows-msvc-relwithdebinfo
cmake --build --preset windows-msvc-relwithdebinfo
cargo test
build\windows-msvc-release\native\vertical_shift.exe --cpu-bench 3000 --seed 42
build\windows-msvc-release\native\vertical_shift.exe --debug-overlay --seed 42
```

## Windows Release package

```powershell
.\scripts\pack_windows_release.ps1
```

Produces `dist\vertical_shift-windows-x64-release.zip` and smokes `--cpu-bench 100 --seed 42`. Overlay off by default in pack runs.

## How systems work (short)

- **Renderer:** Vulkan sprite batcher; gameplay extracts sprites; host draws. Optional GPU timestamps in RelWithDebInfo (`VS_GPU_TIMESTAMPS`).
- **Procgen:** seeded intents → parametric geometry → validate → stream; headless proves reachability on shared movement rules.
- **Memory:** OS heap + fixed effect pool + generational entity IDs (no custom global frame allocator).
- **Tests:** `cargo test` (incl. proptest), C++ `abi_smoke`, pack smoke — see [docs/testing/README.md](docs/testing/README.md).
- **Optimizations measured:** Phase 10 spatial cull + baselines; SIMD/ASM not justified.

## Open limitations

Android not device-verified; Linux/iOS/Win-ARM64 unevaluated as hosts; music deferred; placeholder WAV/TTF; Assembly not shipping.

## Android

See [android/README.md](android/README.md).

## Stack

C++ host · Vulkan · Rust sim · C ABI **1.7** · miniaudio · stb · volk.

## Docs

[Architecture](docs/architecture/overview.md) · [ADRs](docs/adr) · [Baselines](docs/performance/baselines.md) · [Compatibility](docs/compatibility/matrix.md) · [Testing](docs/testing/README.md)
