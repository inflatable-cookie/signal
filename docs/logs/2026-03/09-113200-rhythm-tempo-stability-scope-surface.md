# Rhythm Tempo Stability Scope Surface

Date: 2026-03-09
Owner: core-product

## Summary

Added a public tempo stability scope so Signal can explicitly tell consumers
whether a file is stable end to end, stable with localized edge damage, only
stable through a core region, or unstable through the middle. This turns the
paired span surfaces from the previous batch into a compact consumer-facing
semantic instead of leaving wrappers to infer meaning from raw span coverage.

## Work completed

- added `TempoStabilityScope`, `TempoStabilityScopeSupport`, and
  `TempoStabilityScopeSummary` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- published `TempoDiagnostics.stability_scope`
- propagated the same scope summary into
  `BeatAnalysisResult::tempo_consumption(...)` so compact consumers can read the
  classification without inspecting raw diagnostics
- derived the scope from:
  - `edge_trimmed_stable_span`
  - `stable_core_span`
  - interval-outlier edge locality
  - interior rejection density
- updated `file_rhythm_probe` and `offline_rhythm_demo` to print the new scope
  and support values
- added regression coverage for:
  - whole-track stable click material
  - localized edge damage
  - core-stable-only behavior

## Real-file result

Test file:
`/Users/betterthanclay/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav`

Current result after this batch:

- `bpm=128.00000`
- `tempo_interpretation=SnapInteger/NearIntegerPulse`
- `tempo_state=Lock/StableIntegerTempo`
- `tempo_consumption=.../scope:StableWithLocalizedEdgeDamage`
- `stability_scope=StableWithLocalizedEdgeDamage/edge_trimmed:0.996/contiguous:0.664/interior:0.981/edge_locality:0.971`
- `edge_trimmed_stable_span=beats:0..735/.../coverage:0.996/... trim:0:3 interior:14`
- `stable_core_span=beats:216..706/.../coverage:0.664/... trim:216:32 interior:0`

This is the intended result shape for the Garamond master. Signal still locks
the integer tempo, but it now also says directly that the file is stable with
localized edge damage instead of making consumers infer that from the paired
span diagnostics.

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -q -p signal-analysis-rhythm --example file_rhythm_probe -- '/Users/betterthanclay/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav'`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- `effigy` still needs serial execution on this repo; overlapping `health`,
  `test`, and `validate` runs continue to hit the known workspace lock conflict.
- The current scope surface is intentionally consumer-facing and categorical.
  It does not yet tune the tempo-state action itself based on the scope beyond
  making that scope directly visible on the compact decision result.

## Next Task

Use the new tempo stability scope to tune the tempo-state and tempo-consumption
policy itself, especially deciding when `StableWithLocalizedEdgeDamage` should
still behave like a hard lock versus a guarded lock or monitor path, and when
`CoreStableOnly` should downgrade consumers away from current-tempo lock even if
the refined BPM itself still looks plausible.
