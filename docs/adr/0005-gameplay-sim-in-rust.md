# ADR 0005 — Gameplay Simulation in Rust

## Context

Phase 3 needs entities, gravity, AABB platforms, jump, and a follow camera, while the C++ host already owns Win32 + Vulkan rendering.

## Problem

Where should the simulation live so gameplay stays free of GPU commands and the C ABI stays batch-oriented?

## Alternatives

1. **C++ simulation** next to the renderer — fewer FFI hops; weaker ownership story for later procgen/validation.
2. **Rust simulation** with per-entity FFI — unsafe and slow.
3. **Rust world + batch ABI** (`tick` / `extract`) — matches ADR 0001 and Phase 2 extract architecture.

## Choice

Alternative 3.

- `runtime-core`: specialized storages, AABB, player controller, tutorial level, follow camera.
- `runtime-bridge`: `vs_world_*` C ABI; panics caught at the edge.
- C++ `GameplayScene` + `InputState` only marshal input and convert extract buffers to `SpriteInstance` / `Camera2D`.

## Consequences

- Dual build remains (CMake invokes Cargo).
- Texture GPU handles stay on the host; sim uses logical `texture_id` remapped at extract time.
- Dash/swept collision and procgen stay future work (Phase 4/5).

## Validation

- `cargo test` (landing + jump).
- Playable keyboard controls; `--validation` smoke with idle sim.
