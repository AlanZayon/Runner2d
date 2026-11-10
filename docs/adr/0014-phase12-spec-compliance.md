# ADR 0014 — Phase 12 Spec Compliance

## Context

Phases 0–11 delivered a playable Windows x64 + Vulkan runner (`0.11.0-phase11`, ABI 1.6) but an audit against `complete plan.md` §53 / §16 / §18 / §33–§35 / §46 / §48–§49 found gaps: no animation system, no debug overlay/draw, no quality profiles, no GhostChallenge, shallow difficulty tiers, no property tests, missing RelWithDebInfo/ASAN presets, missing `docs/testing/`.

## Decision

1. **Scope:** Close Windows-implementable §53 gaps in Phase 12. Product version `0.12.0-phase12`. ABI **1.7** (sprite UV rect + quality/effect caps + debug AABB export).
2. **Remain deferred (honest):** Android device gate, Linux/iOS/Win-ARM64 hosts, 2nd graphics backend, music streaming, SIMD/Assembly shipping.
3. **Ship:** animation clips, F3 overlay + AABB debug draw (off by default in Release pack), quality Low→Ultra + hysteresis, GhostChallenge + director tiers, swept AABB, save v1→v2, proptest, input replay, Vulkan timestamp queries in non-Release-pack builds.

## Consequences

- README §55 expanded; matrix links Win-ARM64 evaluation checklist.
- CMake presets: RelWithDebInfo + ASAN (when MSVC supports `/fsanitize=address`).
- No false “supported” claims for unverified platforms.
