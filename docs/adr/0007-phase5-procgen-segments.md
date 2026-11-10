# ADR 0007 — Phase 5 Procedural Segments

## Context

Phase 4 shipped a handcrafted runner course. Phase 5 needs a **reproducible, validated** procedural sequence with seeds, intents, validation, headless proof, and streaming.

An early implementation only picked among nearly identical static templates after a fixed 480px Recovery corridor, so runs looked the same. Generator v2 fixes that with parametric placement.

## Decision

- **SegmentDefinition** (layout data) is separate from live entities.
- **XorShift64** provides a deterministic RNG stream from an explicit `u64` seed.
- **Parametric generation** (`generate.rs`): each intent draws gap / hazard / coin parameters inside safe ranges, then emits geometry (complete-plan pipeline: intent → constraints → geometry → obstacles → coins).
- **Director** uses a short Recovery intro, a teach spine, then weighted variety slots with **no adjacent duplicate intents**.
- Geometric + **intent-content** validation (e.g. TeachDash must include SolidKill).
- **Headless cue script** (jump/roll/one-shot dash windows from geometry) proves a path on the real `World`.
- **Streamer** keeps current ± ahead/behind segments spawned.
- ABI minor **1.3**: `vs_world_create_seeded`, `VsRunState.{seed,segment_index,segment_count}`.
- `GENERATOR_VERSION = 2`.

## Out of scope

Ghost / SlowFall / SuperStrength intents, Falling Mode, deep multi-path graphs (Phase 6–7).

## Determinism guarantees

Same `GENERATOR_VERSION` + seed → same segment geometry and `course_end` on the same float model. Bit-identical cross-arch floats are not promised.

## Consequences

- Play mode (`vs_world_create`): **random seed** each open; `reset` rolls a **new** seed (segments still continuity-validated).
- Repro mode (`--seed N` / `vs_world_create_seeded`): seed **locked**; reset keeps the same course.
- Variety tests assert mechanic coverage and per-seed hazard layout differences.
