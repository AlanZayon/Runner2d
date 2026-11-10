# ADR 0008 — Phase 6 Ability / Power-up System

## Context

Phase 5 delivered an endless procedural runner. Phase 6 needs temporary abilities (Magnet, Slow Fall, Ghost, Super Strength) with collision categories, QTE-gated Strength duration, CPU effects, and tests — without scattering ability logic across the runtime or inflating per-frame cost.

## Decision

- **AbilityManager** owns timers, activation, expiry, and QTE hand-off. World only orchestrates.
- **MovementModifiers / CollisionModifiers** are derived each tick from active abilities (no permanent player mutation).
- **Pickup = immediate activate** (same type refreshes timer; types coexist). Death/`reset` clears all.
- **CollisionCategory** (`Normal`, `GhostPassable`, `GhostRequired`, `AlwaysSolid`) gates hazard lethality; Ghost is not `collision = false`.
- **Super Strength**: pickup starts **StrengthQte**; rating maps to configurable duration; active Strength grants invulnerability and destroys `Destructible` hazards.
- **EffectPool**: fixed-capacity CPU particles extracted as tinted sprites (cap enforced).
- Sparse **PowerupSpec** / optional GhostPassable / Destructible placement in Recovery/RiskReward; never required to progress.
- ABI minor **1.4**: `VsInputState.qte_pressed`, `VsRunState.{ability_mask,qte_active,qte_prompt}`.
- `GENERATOR_VERSION = 3`.

## Optimization

- Magnet iterates only streamed coins within radius.
- Streamer remains O(window + active); ability slots are fixed (4).
- Particles never allocate beyond `EFFECT_POOL_CAP`.

## Consequences

- Host maps **K/E** to QTE edge presses; logs QTE prompts (polished UI is Phase 8).
- Falling Mode and challenge intents (`GhostChallenge`, etc.) remain later phases.
