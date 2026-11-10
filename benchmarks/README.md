# Benchmarks

## Phase 12 package smoke + replay

```powershell
.\scripts\pack_windows_release.ps1
# unpacks zip and runs: vertical_shift.exe --cpu-bench 100 --seed 42

build\windows-msvc-release\native\vertical_shift.exe --seed 42 --record-replay replay.bin
build\windows-msvc-release\native\vertical_shift.exe --seed 42 --replay replay.bin --cpu-bench 200
```

## Phase 10 (primary)

### CPU-unbound host (no present)

```bat
build\windows-msvc-release\native\vertical_shift.exe --cpu-bench 3000 --seed 42
build\windows-msvc-debug\native\vertical_shift.exe --cpu-bench 3000 --seed 42
```

### Present / vsync smoke

```bat
build\windows-msvc-release\native\vertical_shift.exe --bench --bench-seconds 5 --seed 42
```

### Rust microbench

```bat
cargo bench -p runtime-core --bench sim_bench
```

Results: [docs/performance/baselines.md](../docs/performance/baselines.md) · [ADR 0012](../docs/adr/0012-phase10-optimization.md).

## Historical (Phase 1–2)

```bat
vertical_shift.exe --bench --bench-seconds 5
```

Older `--sprites N` flag was removed after Phase 3 (world extract owns sprite counts).
