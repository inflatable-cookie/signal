# Dense-Event Replica Attribution

Date: 2026-07-13
Roadmap: `g10.029`
Batch: `29.6BW`
Contract: `082`, Rule 30R

## Result

The remaining successor failure is overlap synthesis, not attack timing.

At `2.0x`, the two injected attacks land exactly:

| Attack | Target | Rendered amplitude | Associated peak |
| --- | ---: | ---: | ---: |
| first | 16126 | 1.0 | 16126 |
| second | 16644 | 0.75 | 16382 |

Output `16382` is a third peak with amplitude `0.787177`. It is `256` frames
after the first target and `262` before the second. Because it is louder than
the second real attack, the frozen one-to-one matcher selects it. The reported
timing miss is therefore a real micro-copy exposed by the metric.

The passing successor rows are exact at both attacks. Dense row errors are:

- ordinary: `[[463,401],[219,351],[896,509]]`
- successor: `[[0,0],[0,0],[0,262]]`

## Exclusions

Both anchors attach to exact scheduled outputs. Event reset is present.
Active-owner state is nonempty and assigned. All `49` expected-sample frame
contributions reconstruct their targets with zero real closure error and at
most `6.770e-17` imaginary residue. These stages do not own the first defect.

No renderer policy, threshold, audio, corpus, holdout, stereo, dynamic-ratio,
cache, or routing surface changed. Evidence hash `2336b9773c32b2ca` repeats.

## Decision

Rule 30S owns one bounded event-local overlap proof. Trace the frame-level
contributors at `16382`, retain complementary background overlap, and prevent
neighboring frames from resynthesizing an anchor-owned attack. Do not clamp the
output, weaken the matcher, or move or attenuate either real attack.
