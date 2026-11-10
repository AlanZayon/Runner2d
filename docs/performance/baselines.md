# Performance Baselines

Methodology rules:

- Record real measurements only.
- Note hardware, OS, compiler, flags, validation layers.
- Do not compare across machines without labeling differences.

## BM-00 — Empty Present Loop

**Status:** measured on reference laptop (2026-11-06). Phase 1 clear-only path (superseded for interactive runs by Phase 2 sprite path; kept for history).

### Metrics

| Metric | Value | Unit |
|--------|-------|------|
| Duration | 10.021 | s |
| Frames presented | 603 | count |
| Avg frame time | 16.588 | ms |
| p95 frame time | 18.373 | ms |
| Max frame time | 46.891 | ms |
| FFI `vs_runtime_tick` | 10 | ns/call |
| Working set | 76.05 | MB |
| Validation | off | |

## BM-01 — Textured Sprite Batch

**Status:** measured on reference laptop (2026-11-06). Debug build, FIFO present, single PNG atlas region, same texture for all sprites.

### Command

```bat
vertical_shift.exe --bench --bench-seconds 5 --sprites N
```

### Results

| Sprites | Duration (s) | Presented | Avg ms | p95 ms | Max ms | Draw calls | Working set MB | Validation |
|--------:|-------------:|----------:|-------:|-------:|-------:|-----------:|---------------:|:----------:|
| 1 | 5.026 | 304 | 16.473 | 18.180 | 27.474 | **1** | 76.31 | off |
| 1000 | 3.032 | 184 | 16.392 | 19.147 | 29.992 | **1** | 76.52 | off |

Validation smoke (100 sprites, 3 s, `--validation`): clean (no VUID errors); `draw_calls=1`, avg ≈16.5 ms.

Notes:

- Frame time remains vsync-bound (~60 Hz) from 1→1000 homogeneous sprites on Intel UHD — CPU batch + single draw is not the limiter yet.
- Homogeneous texture batching meets the Phase 2 criterion (1 draw call).

### Environment

| Field | Value |
|-------|-------|
| Date | 2026-11-06 |
| CPU | 12th Gen Intel Core i3-1215U |
| GPU / driver | Intel UHD Graphics / 32.0.101.5763 |
| OS | Microsoft Windows NT 10.0.26200.0 |
| rustc | 1.97.1 (8bab26f4f 2026-07-14) |
| MSVC | 19.44.35228.0 (VS 2022 Build Tools 17.14) |
| CMake config | `windows-msvc-debug` (Ninja, Debug) |
| Vulkan SDK | `C:\VulkanSDK\1.4.363.0` (`glslc` + validation layers) |

## Phase 3 smoke — Idle world (validation on)

**Status:** measured 2026-11-06. Player + 7 platforms extracted; gravity settles; no keyboard input.

| Metric | Value |
|--------|-------|
| Duration | 3.031 s |
| Presented | 183 |
| Avg / p95 frame | 16.478 / 18.795 ms |
| Draw calls | 1 |
| Sprites | 8 |
| Working set | 85.76 MB |
| Validation | on (clean) |
| Command | `vertical_shift.exe --bench --bench-seconds 3 --validation` |

## Phase 4 smoke — Auto-run course (validation)

**Status:** measured 2026-11-06. Handcrafted runner course; idle input (auto-run until hazard); validation layers on.

| Metric | Value |
|--------|-------|
| Duration | 3.023 s |
| Presented | 183 |
| Sim ticks | 180 |
| Avg / p95 frame | 16.412 / 18.736 ms |
| Draw calls | 1 |
| Sprites | 30 |
| Working set | 85.83 MB |
| Validation | on (clean) |
| Notes | Auto-run reached death at ~619 dist (score 61); score logged each second |
| Command | `vertical_shift.exe --bench --bench-seconds 3 --validation` |

## Phase 5 smoke — Seeded procgen (validation)

**Status:** measured 2026-11-06. Generator v2 parametric course (`--seed 42`); validation on.

| Metric | Value |
|--------|-------|
| Seed | 42 |
| Duration | 3.029 s |
| Presented | 183 |
| Sim ticks | 180 |
| Avg / p95 frame | 16.452 / 17.209 ms |
| Draw calls | 1 |
| Sprites (window) | 16 |
| Working set | 85.97 MB |
| Validation | on (clean) |
| Notes | Reached seg 1/8 before idle death; distinct segments streamed |
| Command | `vertical_shift.exe --bench --bench-seconds 3 --validation --seed 42` |

## Phase 10 — Optimization (CPU-unbound + Release)

**Status:** measured 2026-11-10 on reference laptop. Spatial query pad (`COLLISION_QUERY_PAD_X=420`) enabled. See [ADR 0012](../adr/0012-phase10-optimization.md).

### Host Release (`windows-msvc-release`)

| Command | Result |
|---------|--------|
| `--cpu-bench 3000 --seed 42` | wall 0.013 s; avg_sim **0.0031 ms**; avg_extract **0.0012 ms**; WS 73.50 MB |
| `--bench --bench-seconds 5 --seed 42` | avg/p95 frame **16.507 / 18.435 ms** (vsync); avg_sim 0.0233 ms; avg_extract 0.0297 ms; avg_draw+present 0.1536 ms; presented 226 |

### Host Debug (`windows-msvc-debug`)

| Command | Result |
|---------|--------|
| `--cpu-bench 3000 --seed 42` | wall 0.095 s; avg_sim **0.0241 ms**; avg_extract 0.0074 ms |

### Rust `sim_bench` (bench/optimized profile)

```bat
cargo bench -p runtime-core --bench sim_bench
```

| Case | Wall | Notes |
|------|------|-------|
| tick_seed42_3k | 9.638 ms / 3000 ticks ≈ 3.2 µs/tick | collision 24.7%, streamer 24.2% |
| tick_seed7_3k | 16.020 ms / 3000 ticks ≈ 5.3 µs/tick | same mix |
| extract_2k | 0.614 ms / 2000 ≈ 0.27 µs | ~48 sprites |

### Notes

- Frame time remains vsync-bound; sim ≪ 16.7 ms budget on this GPU/CPU.
- SIMD/Assembly **not** applied (extract/collision not vector-bound at this scale).
- Energy/thermal: [energy-checklist.md](energy-checklist.md).
