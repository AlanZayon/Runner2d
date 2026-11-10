# ADR 0009 — Phase 7 Falling Mode

## Context

Phase 6 delivered abilities. Phase 7 needs a special vertical-drop mode (§37) that reuses the existing World — no second engine — with horizontal strafe, vertical camera, shaft obstacles, and a controlled return to auto-run.

## Decision

- **`RunMode`**: `Run | FallingEnter | Falling | FallingExit` owned by `World`.
- **Trigger**: procedural `SegmentIntent::FallingShaft` with `ShaftSpec` (pit bounds, floor, exit lip). Sparse endless append only (not in opening teach buffer).
- **During Falling**: auto-run X off; `move_x * FALLING_STRAFE_SPEED`; gravity + abilities unchanged; camera look-down.
- **Exit**: land on floor pad or `FALLING_MAX_DURATION` → lerp/snap to exit lip at `GROUND_Y` → `Run`.
- Lethal hazards still kill; walls are solid platforms (non-lethal).
- ABI minor **1.5**: `VsRunState.{falling,falling_time}`. Host A/D strafe. `GENERATOR_VERSION = 4`.

## Consequences

- Same streamer / AbilityManager / EffectPool paths.
- Headless opening courses remain shaft-free; endless play can append shafts.
- Polished audio/UI for enter/exit remains Phase 8.
