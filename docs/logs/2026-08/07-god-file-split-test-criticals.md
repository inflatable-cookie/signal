# God-File Split: test/fixture criticals

Status: complete
Created: 2026-08-07
Scope: render-plane unit + offline tests; stretch unit/creative/preview tests;
bridge in_process tests; VST3/CLAP fixtures; sandbox plugin_hosting; LV2 tests;
host-local edge plugin support; realtime_preview_stream_gates

## Baseline

Production high band clear. Remaining criticals were almost entirely
tests/fixtures (render-plane tests 3308, offline tests 1744, stretch tests
1517, bridge tests 1261, VST3/CLAP fixtures, sandbox plugin_hosting, etc.).

## What Changed

### render-plane `src/tests` → topic modules + support
### render-plane `offline/tests` → artifact/bridge/promotion/selectors/parity/…
### stretch `src/tests` → passthrough/HQ/selectors/metrics/…
### stretch `realtime_preview/tests`, `creative/tests`
### stretch `tests/realtime_preview_stream_gates/`
### bridge `in_process/tests/`
### VST3 + CLAP `fixture/` (+ source fragments)
### sandbox `tests/plugin_hosting/`
### LV2 `src/tests/`
### host-local `public_host_edge_plugins/`

Move-only. Assertions unchanged.

## Skipped this batch

- `creative_direct_renewal_dream/tests.rs` (single giant `macro_rules!`)
- `benchmark.rs` (shared measurement surface, not pure tests)
- demos / evidence bins

## After

Test criticals largely cleared. Remaining criticals: dream tests, benchmark,
demo script, evidence corpus-report bin. Some high residue in large test
support/topic files.

## Validation

- render-plane lib 145; stretch lib 194; bridge 21; vst3 26; clap 15; lv2 15
- sandbox plugin_hosting 12; realtime_preview_stream_gates 9
- clippy -D warnings on touched crates

## Next Task

Split dream `macro_rules!` via include! sections; split `benchmark.rs`;
optionally demos/evidence bins or further shrink high test residue.
