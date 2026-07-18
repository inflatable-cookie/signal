# Corrected Direct Objective Gate

Date: 2026-07-19
Batch: 29.7AX
Status: rejected at stereo gate

## Result

The release-only direct mechanics entry gate passes at representation hash
`fdf90f6127749341`, corrected state hash `52d6b8b2bb6edff0`, and corrected
relation hash `425400ebb580b3e1`.

The unchanged synthetic matrix passes at hash `00e522a01b817bb6`:

- structural failures: `0`
- hard channel-mechanics errors: `0/0/0/0`
- state counts, reset/attack/ordinary/unlocked/locked:
  `212983/29274/0/360936/32855`
- borrowed/local locked regions: `3038/5128`
- pending/guidance/output storage high-water: `10/19/7680`
- nonfinite values: `0`
- deterministic repeat: pass

The single corrected stereo run rejects:

- calibrated failures: `38/48`
- improved local windows: `157/384`
- Signal-relative local-row failures: `36/48`
- maximum normalized-Gram residual: `0.7611955347641768`
- structural failures: `0`
- deterministic repeat: pass
- evidence hash: `397128c177d3033e`

Against AU, calibrated failures improve by `2` and improved local windows by
`39`. Local-row failures remain `36/48`; the maximum residual is bit-identical.
The owner-peak correction has a real local effect but does not close the direct
topology's stereo ownership failure.

## Stop

Mono and long-development do not run. No retry, tuning, row repair, listening,
export, concealed read, or holdout access occurred.

## Next Task

Run Batch 29.7AY under Rule 31AB. Freeze AU/AX and perform a no-audio source-to-
code architecture reassessment before another candidate.
