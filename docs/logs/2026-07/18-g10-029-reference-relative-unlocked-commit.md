# g10.029 Reference-Relative Unlocked Commit

Date: 2026-07-18
Batch: 29.7AQ
Status: rejected at stereo gate; topology closed

## Result

The normalized material renderer now has one isolated Rule 31X entry point.
Every channel computes ordinary recurrence unchanged. For `Ordinary` and
`Unlocked`, the greatest-current-energy channel supplies one phase rotation;
all channels apply it to their current coefficient. Lower channel wins an
energy tie. The frozen Rule 31R/31T/31U entry points remain unchanged.

Focused mechanics preserve channel magnitude and current interchannel phase
relation within `1e-12`. The unchanged synthetic gate passes with zero
structural, mechanics, or nonfinite failures. All five state counters execute,
source/output/guidance high-water remains `5/2/19`, repeat is exact, and the
hash is `875b0768ba2066bf`.

The single authorized corrected stereo run rejects:

- calibrated failures: `40/48`
- improved local windows: `125/384`
- Signal-relative local-row failures: `44/48`
- maximum normalized-Gram residual: `0.8700034314389535`
- structural failures: `0`
- evidence hash: `88d9c0f68ea2954b`

Rule 31V recorded `46/48`, `110/384`, and `44/48`. Rule 31X therefore produces
a real local improvement but does not change the row-level failure boundary.
Independent unlocked rotation was one contributor, not the complete missing
waveform invariant.

## Stop

The run stopped at the first objective miss. Mono, long-development, retry,
row repair, tuning, audio export, listening, holdout, dynamic ratio, realtime,
and product work did not run. The candidate is retained as exact rejected
evidence, not promoted.

## Validation

- focused relation-preservation mechanics: pass
- frozen Rule 31R, 31T, and 31U regressions: pass
- Rule 31X synthetic gate: pass, hash `875b0768ba2066bf`
- Rule 31X corrected stereo gate: rejected once as recorded above
- `cargo fmt -p signal-dsp-stretch --check`: pass
- `cargo test -p signal-dsp-stretch`: pass
- release missing-docs check: pass
- `effigy qa:docs`: pass
- `effigy qa:northstar`: pass
- `effigy health`: pass
- `effigy validate`: pass

## Next Task

Re-enter architecture planning. Explain the remaining row-level loss across
the two independently windowed source layers and their waveform-domain sum
before selecting another renderer.
