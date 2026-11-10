# ADR 0013 — Compatibility Fallbacks (Phase 11)

## Context

Phase 11 requires an honest compatibility story (§7.3 / §41 / §49): expand documentation and packaging without claiming universal support. The reference host remains Windows x64 MSVC + Vulkan.

## Decision

| Area | Policy |
|------|--------|
| **Graphics** | **Vulkan only.** No OpenGL ES / Direct3D / Metal path. Missing Vulkan → init fails with a clear log (Android also `android.hardware.vulkan.level` required). A second backend is **not necessary** while Vulkan covers the priority targets (Windows + Android scaffold). |
| **CPU** | **Scalar baseline** for all shipping code. No AVX/NEON requirement. SIMD/ASM remain deferred (Phase 10 measured: not justified). Any future specialized path must keep a scalar fallback + detection (§41). |
| **Audio** | Soft-fail: device/init failure → mute SFX; missing WAV → skip clip. |
| **Fonts** | Soft-fail: missing TTF → HUD/menus without text atlas (gameplay still runs). |
| **Assets** | Soft-fail per loader; critical sprite PNG failure aborts scene init. |
| **Android** | Scaffold only until [device checklist](../android-device-checklist.md) passes. arm64-v8a only in `abiFilters`. |
| **Other OS / arch** | Linux, iOS/macOS, Windows ARM64: **evaluated-deferred** — see evaluation checklists. Not implemented; not claimed. |

## Consequences

- Distribution focuses on a **Windows Release zip** verified on the reference machine.
- Portfolio README lists verified vs open platforms (§55).
- Expanding to Linux/iOS or a second GPU API is a new ADR + matrix row after real builds/tests.
