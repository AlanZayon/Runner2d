# Compatibility Matrix (Phase 12)

Do not claim support without Build + Run evidence on the listed target (§7.3 / §55).

| Target | Build | Run | Packaged | Notes |
|--------|:-----:|:---:|:--------:|-------|
| Windows 10/11 x64 (MSVC) | Yes | Yes | Yes (`scripts/pack_windows_release.ps1`) | **Reference.** Vulkan + Rust ABI 1.7 |
| Windows ARM64 | No | No | No | [Evaluation checklist](win-arm64-evaluation.md) |
| Android arm64-v8a | Scaffold | Untested on reference | APK via Gradle when NDK/SDK present | Device checklist required for “supported” |
| Android x86_64 (emulator) | No | No | No | Future `abiFilters` note only |
| Linux x64 | No | No | No | [Evaluation checklist](linux-evaluation.md) |
| iOS / macOS | No | No | No | [Evaluation checklist](apple-evaluation.md) |

## Reference hardware (verified)

| Item | Value |
|------|-------|
| CPU | 12th Gen Intel Core i3-1215U |
| GPU / driver | Intel UHD Graphics / 32.0.101.5763 |
| RAM | ~8 GB |
| Display | 1366×768 @ 60 Hz |
| OS | Microsoft Windows NT 10.0.26200 |
| Toolchain | VS 2022 Build Tools 17.14, rustc stable, Vulkan SDK 1.4.x |

## CPU / graphics baseline

- Shipping CPU path: **scalar** x86-64 / aarch64 (no ISA extras required).
- Shipping GPU API: **Vulkan** only ([ADR 0013](../adr/0013-compatibility-fallbacks.md)).
- Soft-fail: audio mute, missing optional font; hard-fail: no Vulkan / missing player sprite.

## Explicit non-claims

- No universal OS/CPU/GPU compatibility.
- Android is **not** “supported” until a physical device passes [android-device-checklist.md](../android-device-checklist.md).
- Linux / Apple / Win-ARM64 are **not** supported until built, installed, and recorded here.
- Assembly / SIMD: not shipping ([ADR 0012](../adr/0012-phase10-optimization.md)).
