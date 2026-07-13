# g10.029 Adaptive Single-Frame Synthetic Quality Gate

Date: 2026-07-13

## Scope

Batch 29.6BS runs the Rule 30N synthetic gate on the frozen ordinary and
combined event-plus-vertical modes. It changes no study, geometry, schedule,
peak, event, vertical, or phase policy and reads no corpus or holdout audio.

## Result

The candidate is rejected.

- `12` controls at identity, `0.75`, `1.5`, and `2.0`; `48` cases and `96`
  ordinary/combined renders per review
- exact target length, full coverage, zero fill/fade actions, coefficient and
  magnitude identity, silence, finiteness, symmetry, and exact repeat pass
- identity peak/RMS error is `1.140e-11/5.714e-12`
- maximum imaginary residue is `3.217e-13`; symmetry error is zero
- maximum reported output-frame condition is `4.941683`
- `25` hard pitch or event-placement failures and one combined-only regression
- maximum tone error is `6.842e-4` radians/sample against `1e-6`
- isolated-event error is `496` frames against `1`
- dense one-to-one event error is `896` frames against `256`
- maximum post-attack replica ratio is `18.389432`
- maximum absolute reported texture/mode delta is `15.685272`
- evidence hash `6781d49348dfa931` repeats exactly

The independent frequency-measurement control resolves exact `55`, `440`, and
`8000 Hz` tones at every tested output length within `2.5e-7` radians/sample,
so the pitch rejection is not a resolution artifact.

## Attribution Boundary

Ordinary transport already fails low-tone pitch, stretched mid/high-tone rows,
all stretched isolated-event rows, and several dense rows. Combined phase
policy therefore cannot own the general failure. It improves dense placement
on several rows, but the `2.0` combined dense row still misses by `259` frames
and the `0.75` mid-tone row becomes the one combined-only hard regression.

This is enough to stop before the mono development corpus, not enough to choose
a redesign. Rule 30O must trace expected and transported phase advance plus
event-local synthesis contributions on the frozen failing rows.

## Next Task

Execute Batch 29.6BT under Rule 30O. Keep algorithm changes, corpus, holdout,
listening, tuning, linked stereo, dynamic ratio, and product routing closed.
