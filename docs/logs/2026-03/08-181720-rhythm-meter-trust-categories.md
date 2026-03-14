# Rhythm Meter Trust Categories

Date: 2026-03-08
Owner: core-product

## Summary

Extended the public rhythm meter surface with a Signal-owned trust layer so
downstream consumers can distinguish stable whole-track meter, sustained meter
recovery, and weaker tentative meter claims without reimplementing threshold
logic in product code.

## Work completed

- added `MeterTrustLevel` to `signal-analysis-rhythm` with:
  - `Stable`
  - `Recovering`
  - `Tentative`
- extended `MeterEstimate` so every promoted meter claim now carries a trust
  category alongside:
  - detection provenance
  - support profile
  - confidence breakdown
  - recovery context
- added trust calibration logic that maps:
  - strong whole-track support to `Stable`
  - sustained segment-backed late recovery to `Recovering`
  - weaker promoted meter claims to `Tentative`
- updated `infer_meter(...)` so trust is assigned after support-profile
  construction for whole-track candidates and directly during segment-recovery
  candidate creation
- updated the offline rhythm demo to print `meter_trust`
- expanded the rhythm calibration tests so the public Signal contract is
  explicit:
  - structured active four-four is `Stable`
  - sustained recovery after destabilized sections is `Recovering`
  - weak backbeat promotion stays `Tentative`

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- Effigy runs were kept serial again to avoid the known workspace lock conflict
  when overlapping repo-owned tasks.
- The trust mapping is intentionally conservative: only strong whole-track
  claims become `Stable`, while segment promotions are surfaced as
  `Recovering` or `Tentative` instead of being overstated as globally stable.

## Next Task

Add an explicit meter-action or consumption recommendation surface above trust,
such as whether a caller should lock, monitor, or defer meter-dependent
features, then calibrate that recommendation against stable, recovering, and
tentative families so Finch can consume Signal-owned meter behavior directly.
