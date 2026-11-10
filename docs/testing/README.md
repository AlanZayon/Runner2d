# Testing suite

## Rust unit + headless

```bat
cargo test
```

Covers math/AABB, abilities, QTE, director seeds, streamer, Falling Mode, GhostChallenge, swept tests, property tests (`proptest`).

## C++ smoke

```bat
cmake --preset windows-msvc-debug
cmake --build --preset windows-msvc-debug
ctest --preset windows-msvc-debug --output-on-failure
```

Includes `abi_smoke` and `camera_math_test`.

## Host CPU / present benches

```bat
build\windows-msvc-release\native\vertical_shift.exe --cpu-bench 3000 --seed 42
build\windows-msvc-release\native\vertical_shift.exe --bench --bench-seconds 5 --seed 42
```

## Pack smoke (Phase 11+)

```powershell
.\scripts\pack_windows_release.ps1
```

Unpacks zip and runs `--cpu-bench 100 --seed 42`.

## Input replay

```bat
vertical_shift.exe --seed 42 --record-replay replay.bin
vertical_shift.exe --seed 42 --replay replay.bin --cpu-bench 500
```

## Sanitizer (optional)

```bat
cmake --preset windows-msvc-asan
cmake --build --preset windows-msvc-asan
```

See [../toolchain.md](../toolchain.md). If ASAN configure fails on Build Tools-only installs, use Debug + careful review instead.
