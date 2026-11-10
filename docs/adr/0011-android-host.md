# ADR 0011 — Phase 9 Android host

## Context

Phase 8 delivered Windows audio/UI/save. The complete plan §7.2 / Fase 9 requires a native Android host: touch, lifecycle, surfaces, portrait, and physical-device verification. The sim/ABI stay shared; only the host layer ports.

## Decision

1. **Native shell:** `android.app.NativeActivity` + NDK `android_native_app_glue` driving `android_main` (Game Activity AAR is a declared dependency for a future Java shell swap; native loop is glue-compatible).
2. **ABI filter:** `arm64-v8a` only.
3. **Vulkan:** `VK_KHR_android_surface` + `ANativeWindow`; `lose_window_surface` / `recreate_window_surface` on TERM/INIT.
4. **Touch:** host `GestureRecognizer` (swipe up/down/right, tap) → `VsInputState`; mouse-drag simulates gestos on Win32 (§20).
5. **Assets:** `AssetFs` (File / AAssetManager); SPIR-V + WAV/PNG/TTF packaged under APK `assets/`.
6. **Save:** `SaveStore` override dir = `internalDataPath`.
7. **ABI version:** remains **1.6**; product string `0.9.0-phase9`.
8. **Support gate:** Android is not “supported” until installed and tested on a **physical device** (§7.2). Emulator smoke is intermediate.

## Consequences

- Windows host remains the daily-driver reference build.
- Gradle + `scripts/build_android.ps1` orchestrate Rust aarch64, shaders, and APK.
- Phase 10 owns energy/thermal/profiling; Phase 11 owns extra ABIs / store distribution.
