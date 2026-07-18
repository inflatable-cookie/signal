# g10.029 Direct Scale Objective Gate

Date: 2026-07-19
Batch: 29.7AU
Status: rejected at stereo gate

## Result

The direct renderer implements the frozen absolute source projection, fixed
ten-tick coefficient ring, nineteen-tick joint-magnitude ring, Rule 31V
guidance, one direct state commit, exclusive per-scale inverse synthesis, and
same-channel bounded output overlap.

The no-audio entry gate preserves representation hash `fdf90f6127749341` and
state hash `430543f8e1dce721`. Synthetic evidence passes at hash
`00e522a01b817bb6`:

- structural failures: `0`
- hard channel-mechanics errors: `0/0/0/0`
- state counts, reset/attack/ordinary/unlocked/locked:
  `212983/29274/0/360936/32855`
- borrowed/local locked regions: `3038/5128`
- pending/guidance/output storage high-water: `10/19/7680`
- nonfinite values: `0`
- deterministic repeat: pass

The single corrected stereo run rejects:

- calibrated failures: `40/48`
- improved local windows: `118/384`
- Signal-relative local-row failures: `36/48`
- maximum normalized-Gram residual: `0.7611955347641768`
- structural failures: `0`
- deterministic repeat: pass
- evidence hash: `af461c9576729c4e`

All `118` improved windows are image controls. Tone controls improve `0/192`
windows; their ratio groups reach maximum residuals `0.558974083`,
`0.536116668`, and `0.761195535` at `0.75`, `1.5`, and `2.0`. The topology
improves local image consistency but fails tonal phase ownership.

## First-Owner Suspect

Compatible locked borrowing currently computes each channel's atom phase as
the borrowed trajectory plus that atom's offset from the same channel's peak.
At the peak itself the offset is zero for every channel, collapsing all peak
phases onto the borrowed trajectory. Existing mechanics fixtures prove within-
channel peak-relative shape but do not test inter-channel phase at the peak.

This is attribution, not a correction. Batch 29.7AV must prove or refute the
collapse analytically before changing state code or rerunning a candidate.

## Stop

Mono and long-development do not run. No retry, tuning, row repair, audio
export, listening, concealed read, or holdout access occurred.

## Next Task

Run Batch 29.7AV as attribution-only. Freeze AU evidence and test compatible
locked-peak inter-channel phase ownership with one analytic fixture.
