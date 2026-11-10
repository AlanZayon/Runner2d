# ADR 0010 — Phase 8 Audio, UI, Persistence

## Context

Phase 7 delivered Falling Mode and a complete endless sim (`0.7.0-phase7`, ABI 1.5). The host still logged run state only — no SFX, HUD, menus, or durable high scores. Phase 8 needs product-facing feedback without a generic UI/audio framework (§27/§28/§40).

## Decision

1. **Events ABI (1.6):** Sim accumulates `GameEvent` in a 32-slot ring (`events.rs`); host drains via `vs_world_drain_events`. Kinds cover jump/land/dash/roll/coin/death/ability/falling/QTE/destroy. Version string `0.8.0-phase8`.
2. **Audio:** Vendored **miniaudio** + `vs_audio` (`AudioDevice` + `AudioMixer`). WAV placeholders in `assets/audio/`. Event → clip map; missing files fail soft (mute). Master/SFX volumes; music stub deferred.
3. **UI/fonts:** **stb_truetype** glyph atlas → RGBA texture; `UiDraw` emits `SpriteInstance` into the same Vulkan sprite batch. Screens: `Title | Playing | Paused | GameOver`; HUD + QTE overlay during play. No toolkit.
4. **Persistence:** `SaveStore` writes versioned JSON (`vertical_shift_save.json` under `%APPDATA%/VerticalShift/` or cwd). Fields: best score/distance, volumes. Not coupled to `World`.
5. **Feedback:** Host camera shake from Death/Dash/Destroy events (existing `Camera2D.shake_offset`).

## Consequences

- Host owns screen flow; sim auto-restart on death is suppressed so GameOver can show.
- Android / touch UI remains Phase 9.
- Font asset `assets/fonts/ui.ttf` is a placeholder TTF suitable for stb; swap for a libre face when packaging.
