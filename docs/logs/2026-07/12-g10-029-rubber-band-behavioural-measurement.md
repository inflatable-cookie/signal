# Rubber Band Behavioural Measurement

Date: 2026-07-12
Roadmap: `g10.029`
Batch: `29.6BE`
Status: complete; attribution ready

## Gate

- expected rows: `264`
- measured rows: `264`
- exact-length rows: `264`
- finite rows: `264`
- unclipped rows: `264`
- repeat-identical rows: `264`
- required CLI modes: `5/5`
- tool/version: `/opt/homebrew/bin/rubberband`, `4.0.0`
- direct public-state adapter: unsupported with explicit receipt

Generated WAV files and TSV reports remain under
`target/rubber-band-behavioural-probe-v3/`. They are reproducible build evidence,
not checked-in source artifacts.

## Raw Signatures

The isolated impulse shows large mode-dependent local displacement despite
exact final duration. At `1.5x`, signed peak offsets are `-578` frames for R2
default, `-577` for R2 no-reset, `-578` for R2 no-lamination, `-101` for R3
standard, and `-168` for R3 short. At `1.25x`, R3 standard and short differ by
`54` frames (`-59` versus `-5`).

Dense-event measurement now uses non-overlapping event neighbourhoods. Identity
places both events exactly in every mode. At `1.25x`, mean absolute offsets are
`92` frames for every R2 contrast, `59` for R3 standard, and `107` for R3
short. At `0.75x`, they are `63.5`, `35`, and `43.5` frames respectively.

The R2 no-lamination impulse output retains an impulse-like `24.099 dB` crest
and zero post-event replica ratio across non-identity ratios while its vertical
phase-coherence proxy diverges sharply from R2 default. R2 no-reset changes
crest and replica behavior without consistently changing the isolated impulse
location. R3 standard and short retain distinct event, texture, and vertical
coherence signatures.

These are observations, not final mechanism attribution. Boundary and soft
onset rows have larger detector uncertainty and must not dominate Batch 29.6BF.

## Measurement Corrections

The first internal pass exposed three invalid measures and was discarded:

- soft onsets used sample peaks instead of local energy rise
- replica ratios referenced projected rather than measured event peaks
- dense events could claim the same peak through overlapping search windows

The final report uses energy-rise refinement for the soft onset, measured event
peaks for crest/replicas, and disjoint event neighbourhoods. Vertical coherence
is a windowed adjacent-bin phase-relation statistic, not adjacent-sample
correlation.

## Next Task

Run Batch 29.6BF attribution. Require direction agreement across relevant
families and ratios. Do not infer R3 internals or implement Signal synthesis.
