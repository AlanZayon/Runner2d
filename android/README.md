# Android host (Phase 11)

NativeActivity + `libvertical_shift.so` (**arm64-v8a** only). Portrait locked. Touch gestures via host recognizer.

Scaffold aligned to product version **0.12.0-phase12**. This is **not** a supported platform until a physical device passes [../docs/android-device-checklist.md](../docs/android-device-checklist.md).

## One-time setup

1. Install Android SDK + NDK (API 26+), set `ANDROID_HOME` / `ANDROID_NDK_HOME`.
2. `rustup target add aarch64-linux-android`
3. Copy `local.properties.example` → `local.properties` with `sdk.dir=...`
4. **Gradle wrapper:** this tree does not commit `gradlew` / `gradlew.bat` unless generated locally. From `android/` with Gradle on `PATH`:

```bat
gradle wrapper --gradle-version 8.7
```

Commit the generated `gradlew*`, `gradle/wrapper/*` when available so CI/devs can build without a system Gradle install.

## Build (debug)

From repo root:

```powershell
.\scripts\build_android.ps1          # shaders + Rust aarch64 (+ gradle if wrapper present)
cd android
.\gradlew.bat :app:assembleDebug
```

APK: `android/app/build/outputs/apk/debug/`.

## Release (unsigned)

Unsigned release APK for sideload / store signing later:

```powershell
cd android
.\gradlew.bat :app:assembleRelease
```

Output: `android/app/build/outputs/apk/release/`. Sign with your keystore before Play Store submission (out of Phase 11 scope).

## ABI notes

- Shipping filter: `arm64-v8a` only.
- `x86_64` emulator ABI is a future `abiFilters` addition — not built today.

## Device verification

[../docs/android-device-checklist.md](../docs/android-device-checklist.md) · matrix: [../docs/compatibility/matrix.md](../docs/compatibility/matrix.md)
