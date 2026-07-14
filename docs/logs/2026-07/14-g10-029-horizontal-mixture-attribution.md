# Horizontal Mixture Attribution

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CO
Scope: trace-only isolated/mixed synthetic attribution

## Decision

Choose predictor-equation correction, not observation-geometry redesign.

All four isolated tones already create frame-rate sidebands above the frozen
`-60 dB` ceiling. Mixing increases nearest-bin auxiliary-ratio variance but is
not required for failure. The next batch corrects one source-verified
translation error in preliminary horizontal energy scaling.

## Evidence

| Tone | Isolated OOB | Ratio variance | Mixed ratio variance | Output-phase variance | Spur offset | Hash |
| ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `110 Hz` | `-26.555010 dB` | `1.710e-7` | `2.558e-5` | `3.371e-6` | `33.339844 Hz` | `60c704b7d391eac0` |
| `164.8138 Hz` | `-37.758329 dB` | `3.819e-9` | `3.179e-5` | `1.314e-9` | `33.428388 Hz` | `90d634c4acf0ed62` |
| `220 Hz` | `-23.544808 dB` | `7.479e-10` | `6.744e-5` | `6.892e-7` | `33.476563 Hz` | `dc7d9193ad4fab48` |
| `329.6276 Hz` | `-51.499468 dB` | `5.789e-11` | `8.651e-7` | `5.373e-11` | `33.165369 Hz` | `2eb4b1e9d064f9ae` |

Mixed horizontal output remains `-28.182097 dB`, hash
`6ae9748d411e5b8f`. Mixed auxiliary-ratio variance exceeds isolated variance
for every tone, but every isolated output already fails. Each isolated dominant
spur is within `0.168 Hz` of one `33.333333 Hz` frame-rate offset.

Nearest-bin ratio and output-phase variance can be extremely small while the
full synthesized support remains dirty. The defect is not a fixed nearest-bin
pitch error. It lies in the full horizontal state/equation attached to the
output grid.

## Source Reinspection

Pinned Signalsmith Stretch `1.3.2` does not target-normalize its preliminary
horizontal state. It computes the prior-output/current-input/conjugated-
auxiliary product, then divides by the larger of previous and current input
energy plus its floor. Separate vertical re-prediction later normalizes to
target energy.

Signal's proof used direct target normalization in both stages. This changes
the relative preliminary-bin magnitudes which weight the later vertical phase
sum. The nominal per-bin rotations visible in the specimen cancel algebraically
for Signal's unmapped same-bin mono case; they do not explain this fixed-ratio
failure.

No upstream expression transfers into Signal. The finding corrects topology,
not a parameter.

## Closed Lanes

- windows, transform size, interval, distance, weight, and floor changes
- observation-geometry redesign
- corpus, holdout, listening, stereo, and dynamic ratio
- cache and production routing

## Next Task

Run Batch 29.6CP. Replace only preliminary horizontal target normalization with
the previous/current input-energy denominator. Preserve final vertical target-
energy normalization and every other frozen mechanism. Rerun the complete
Rule 31G synthetic gate and CN/CO attribution before any real audio.
