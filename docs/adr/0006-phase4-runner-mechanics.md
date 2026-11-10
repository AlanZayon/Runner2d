# ADR 0006 — Phase 4 Runner Mechanics

## Context

Phase 3 delivered free platforming. Phase 4 needs a short playable runner sequence: auto-run, dash, roll, hazards, coins, score, restart.

## Choice

Extend the existing Rust `World` tick pipeline rather than a new crate:

- Auto-run along +X with light strafe.
- `PlayerState` FSM (Running/Jumping/Falling/Dashing/Rolling/Dead).
- `Hazard` / `Coin` components; dash ignores SolidKill; roll required for LowBar.
- Score via `VsRunState`; death + R / auto-restart via `World::reset`.
- Handcrafted `spawn_runner_course` (safe spawn corridor for idle smoke).

## Consequences

- ABI minor bumped to 1.2 (`dash_pressed`, `roll_held`, `restart_pressed`, `vs_world_get_run_state`).
- Particles/UI fonts deferred; tint communicates dash/roll/death.
- Procgen remains Phase 5.

## Validation

`cargo test` covers dash/roll/coins/hazards/finish/reset; host `--validation` smoke.
