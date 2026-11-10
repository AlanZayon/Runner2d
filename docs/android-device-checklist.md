# Android physical-device checklist (Phase 9)

Android is **not** marked supported until every item passes on a real device (§7.2).

1. Build APK (`scripts/build_android.ps1` then `android/gradlew :app:assembleDebug`).
2. Install via `adb install -r app-debug.apk`.
3. Launch → Title shows; **tap** starts run.
4. Play ≥ 1 minute: swipe up jump, down roll, right dash; coins/SFX audible.
5. Press **Back** → Pause; tap → resume.
6. Home / app switch → resume without crash; surface recreated.
7. Confirm **portrait** lock (no landscape layout).
8. Die → GameOver → tap restart; best score persists across kill/relaunch.
9. Note device model / API level in [compatibility/matrix.md](compatibility/matrix.md).
