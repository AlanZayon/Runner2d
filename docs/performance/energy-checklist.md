# Energy / thermal checklist (Phase 10)

Do **not** claim low power without measurements (§45).

## Windows proxy (always record with benches)

- Working set MB from `PHASE10 RESULTS`
- Wall time for `--cpu-bench` and `--bench`
- Note: process energy APIs are optional; WS + duration are the Phase 10 minimum

## Android (physical device)

1. Install Release/debug APK from Phase 9 scaffold.
2. Play **10+ minutes** continuous; note thermal comfort / frame dips.
3. Background/resume mid-run; confirm no runaway CPU when paused.
4. Record device model, API, battery % drop if available.
5. Adaptive quality (cut particles/UI first) only if throttling is observed — document in baselines.
