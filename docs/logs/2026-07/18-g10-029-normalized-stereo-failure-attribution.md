# g10.029 Normalized Stereo Failure Attribution

Date: 2026-07-18
Batch: 29.7AP
Status: complete; one integration correction promoted

## Result

One deterministic replay of the frozen `48` Rule 31V development rows ran at
evidence hash `24cdad83bf3ddeeb`. The observer is audio-inert: attributed and
ordinary renders are sample- and hash-identical.

The two outer source layers are distinct windowed coefficient fields, so their
relation differs from the selected dominant current coefficient. That is not a
state mutation. The first actual operator divergence is the terminal state
commit.

- all `96` retained first/worst state events are `Unlocked`
- all `96` are interior events
- `90/96` have no owner switch
- both tone and image controls fail in the same place
- `0.75x`, `1.5x`, and `2.0x` all fail in the same place
- long and middle scales are represented
- state-commit and layer-projection residuals match exactly in all `96` pairs
- maximum inverse-slice normalized-Gram residual is `0.6423375599950403`
- maximum outer-overlap residual is `0.6205436585782236`

Inverse synthesis exposes the committed damage. Outer overlap sometimes
raises it and sometimes lowers it. Neither is the first owner. Boundary
handling and owner switching are also excluded by the interior, stable-owner
events.

## Architecture Decision

Promote one reference-relative unlocked commit under Rule 31X. Keep every
channel's ordinary recurrence as a precursor. For `Ordinary` and `Unlocked`,
select the greatest-current-energy channel per atom, derive its ordinary phase
rotation, and apply that rotation to every channel's current coefficient. This
retains peer magnitude and current complex relation.

Reset, attack, locked trajectories, classifier, medians, thresholds, geometry,
source schedule, layer projection, inverse synthesis, and overlap remain
unchanged. No external expression or numeric policy transfers.

## Evidence Boundary

The corrected stereo objective gate was not rerun. No renderer correction,
row repair, tuning, audio export, listening, holdout, mono, long-development,
dynamic-ratio, realtime, or product work occurred.

Generated evidence remains ignored under
`target/stretch-normalized-material-attribution/first-divergence.tsv`.

## Validation

- one Rule 31W `48`-row replay: pass, hash `24cdad83bf3ddeeb`
- attributed/ordinary sample and hash parity: pass
- Rule 31R, Rule 31T, and Rule 31U focused regressions: pass
- `cargo test -p signal-dsp-stretch`: pass
- release missing-docs check: pass
- package formatting and diff check: pass
- `effigy qa:docs`: pass
- `effigy qa:northstar`: pass
- `effigy health`: pass
- `effigy validate`: pass

## Next Task

Run Batch 29.7AQ under Rule 31X. Implement the one reference-relative unlocked
commit, prove exact mechanics and relation preservation, then execute the
failure-first synthetic and corrected stereo sequence once.
