# Architecture Overview — Phase 12

```text
Windows x64 MSVC + Vulkan  →  reference (Build / Run / Packaged)
Spec compliance            →  animation, overlay, quality, GhostChallenge, swept, proptest, replay
Android arm64 scaffold     →  hygiene; device gate still open
Linux / Apple / Win-ARM64  →  evaluation checklists only
Graphics                   →  Vulkan only (ADR 0013)
CPU                        →  scalar baseline; no shipping SIMD/ASM
ABI                        →  1.7 (quality + debug AABBs)
```

## Compatibility stance

- Claim support only for platforms with Build + Run evidence ([matrix](../compatibility/matrix.md)).
- Portable Windows Release zip: `scripts/pack_windows_release.ps1`.
- Gap closure: [ADR 0014](../adr/0014-phase12-spec-compliance.md).

## Performance (carried from Phase 10)

- Measure first; sim on reference Windows is ≪ 16.7 ms after spatial cull.
- No shipping SIMD/Assembly — [ADR 0012](../adr/0012-phase10-optimization.md).
- Quality profiles Low→Ultra cap secondary particles; essentials stay sharp.

## See also

- [ADR 0010](../adr/0010-audio-ui-persistence.md) · [ADR 0011](../adr/0011-android-host.md) · [ADR 0012](../adr/0012-phase10-optimization.md) · [ADR 0013](../adr/0013-compatibility-fallbacks.md) · [ADR 0014](../adr/0014-phase12-spec-compliance.md)
- [Baselines](../performance/baselines.md) · [Testing](../testing/README.md) · [Compatibility](../compatibility/matrix.md)
