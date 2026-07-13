# g10.029 Successor Synthetic Quality Gate

Date: 2026-07-13

## Scope

Batch 29.6BV routes the Rule 30P successor through all `48` frozen Rule 30N
control/ratio rows. The ordinary renderer remains the only ablation. Controls,
measurements, limits, onset detection, owner matching, frame geometry, event
reset, and phase policy stay fixed. Corpus and holdout audio remain unread.

## Result

The successor is rejected on one hard check.

- hard failures: `1`
- regressions against ordinary: `0`
- failing row: `DenseEvent`, `2.0x`
- dense peak errors: `[0, 262]` frames; limit `256`
- maximum tone error: `8.211e-7` radians/sample
- maximum isolated-event error: `0` frames
- identity peak/RMS error: `9.992e-16` / `2.144e-16`
- maximum condition: `4.941683`
- maximum symmetry error: `0`
- maximum imaginary residue: `2.734e-13`
- maximum crest: `27.101174 dB`
- maximum replica ratio: `1.287973`
- maximum texture fields:
  `[0.340237494, 0.112228383, 0.108063247, 2.358472210, 0.420652237, 0.117816591]`
- maximum absolute mode deltas:
  `[11.832508535, 18.281792243, 0.566560285, 0.476492114, 0.590069465, 0.551220817]`
- evidence hash: `c72c005d0cd44e3e`

Frozen predecessor hashes remain unchanged: quality `6781d49348dfa931`,
attribution `ddca308a7f60f39e`, ownership `a2d3fb95545cb47f`.

## Decision

Do not relax the dense-event limit or spend the six-frame miss. Exact anchor
detection and frame attachment already pass. The remaining question is why the
one-to-one output metric selects a later second peak: event phase reset,
active-owner transport, overlapping diagonal-dual contributions, or peak
association.

Frozen mono comparison remains closed. No DSP policy, threshold, corpus,
holdout, listening, stereo, dynamic-ratio, cache, or routing work opens.

## Next Task

Execute Batch 29.6BW under Rule 30R. Freeze the dense rows and trace both exact
anchors through event/owner state, overlapping synthesis contributions, local
output peaks, and metric association. Select the earliest owning stage before
redesign.
