# ADR 0012 — Phase 10 Optimization

## Context

Baselines through Phase 5 were Debug + vsync-bound (~16.5 ms) and did not isolate sim CPU. Phase 10 requires measure → hotspot → justified change (§4.1 / §42 / §54), with Assembly deferred unless measured.

## Decision

1. **Instrumentation:** `TickProfile` accumulators in `World::tick` / `extract_sprites`; host timers (`sum_sim_ms`, `sum_extract_ms`, `sum_draw_ms`); `--cpu-bench N` for present-free loops.
2. **Microbench:** `cargo bench -p runtime-core --bench sim_bench` (seeded ticks + extract).
3. **Hotspot opt:** `COLLISION_QUERY_PAD_X` (420 px) culls platforms/hazards/coins/powerups outside the player X window before AABB work. Profile still shows collision ~25% of sim time (necessary work), with streamer ~24%.
4. **SIMD / Assembly:** Extract avg ~0.3 µs — not a vector hotspot. **No SIMD and no Assembly** shipped; revisit only if a future profile shows a dense numeric loop.
5. **ABI:** remains **1.6**; version string `0.10.0-phase10`.

## Measured (reference laptop, 2026-11-10)

| Path | Result |
|------|--------|
| `sim_bench` tick_seed42_3k | wall 9.6 ms / 3000 ticks ≈ **3.2 µs/tick**; collision 24.7%, streamer 24.2% |
| `sim_bench` extract_2k | ≈ **0.27 µs**/extract (~48 sprites) |
| Release `--cpu-bench 3000 --seed 42` | avg_sim **0.0031 ms**, avg_extract **0.0012 ms**, WS ~73.5 MB |
| Release `--bench --bench-seconds 5 --seed 42` | avg frame **16.5 ms** (vsync); avg_sim **0.023 ms** ≪ frame |
| Debug `--cpu-bench 3000 --seed 42` | avg_sim **0.024 ms** |

Conclusion: on the reference Windows host, **present/vsync dominates**; sim is comfortable under 60 Hz budget after the spatial cull.

## Consequences

- Functional tests remain the regression gate.
- Mobile energy/thermal still require a physical device ([energy checklist](../performance/energy-checklist.md)).
- Phase 11 owns broader platform/compat work, not further speculative micro-opts.
