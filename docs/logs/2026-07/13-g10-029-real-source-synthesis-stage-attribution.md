# Real-Source Synthesis-Stage Attribution

Date: 2026-07-13
Roadmap: `g10.029`
Batch: `29.6BZ`
Status: ordinary adaptive synthesis owns the dominant regression

## Result

The real-source defect appears before active-peak tracking, transient anchors,
or event-local overlap ownership. The current-to-ordinary-adaptive transition
owns the broad regression.

| Transition | Timing | Replica | Static residual | Formant residual |
| --- | ---: | ---: | ---: | ---: |
| current → ordinary adaptive | `8/9` | `7/9` | `9/9` | `9/9` |
| ordinary → tracked/no-anchor | `2/9` | `3/9` | `1/9` | `3/9` |
| tracked/no-anchor → tracked/anchor | `3/9` | `4/9` | `7/9` | `3/9` |
| tracked/anchor → event-owned | `0/9` | `0/9` | `0/9` | `0/9` |

Mean transition deltas use lower-is-better fields in the same order:

- current → ordinary: `+196.166667`, `+0.116000`, `+0.084362`, `+0.048668`
- active tracking: `-170.111111`, `+0.104278`, `-0.016404`, `-0.007222`
- anchors: `+35.111111`, `-0.035072`, `+0.013803`, `-0.000071`
- overlap ownership: all zero

Active tracking is useful: it repairs most mean timing loss and some spectral
and formant damage. It does not repair the foundational ordinary stage and it
adds replica energy. Anchors trade lower replicas for worse timing and static
residual. Event-local ownership is inert on all nine rows.

## Integrity

Seven ordinary-adaptive renders breach the existing `7 dB` endpoint-energy
limit. The failing rows are `L004`, `L005`, `L007`, `L008`, `L010`, `L013`,
and `L014`, with endpoint deltas from `7.845183` to `9.784592 dB`. Current and
all tracked stages pass. All modes retain exact length and finite samples.

Adjacent-stage output changes are `[9,9,8,0]` rows. The last zero proves the
bounded synthetic overlap repair does not explain the real-source regression.

## Frozen Evidence

The local report is
`target/stretch-successor-bz-stage-attribution.tsv`. It contains five modes
for each of the unchanged nine development rows and the complete Rule 30T
measurement field set.

- rows: `9`
- modes: `5`
- renders: `45`
- holdout reads: `0`
- listening exports: `0`
- event-fallback renders: `26`
- report SHA-256:
  `064703b05d84fd94d4f9258878efe5a4792c7ccb49aab2ff4bbf1c2dec579fd7`
- manifest hash: `59fde9d5897fe070`
- render hash: `43806ef3d1b3a311`
- measurement hash: `30b29a8a65b50861`
- aggregate hash: `557eaf8e6c9ee5c5`

## Decision

Do not tune tracking, anchors, or overlap ownership. The next attribution must
split the ordinary adaptive stage itself. Fixed `512`, `1024`, `2048`, and
`4096` controls exhaust the existing window bank and decide whether one time
resolution, adaptive transitions, or the shared phase/output lattice owns the
damage.

The development-report implementation was also split into measurement and
orchestration modules. This removes the god-file finding introduced by Batch
29.6BY without changing its frozen hashes.

## Next Task

Execute Batch 29.6CA under Rule 30V. Keep holdout, listening export, tuning,
detector/schedule policy, linked stereo, dynamic ratio, cache, and routing
closed.
