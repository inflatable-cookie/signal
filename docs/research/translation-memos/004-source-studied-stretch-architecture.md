# Source-Studied Stretch Architecture

Status: historical prototype rejected; topology distinction superseded by memo 019
Memo: `g10.029` source-study reset
Owner: dsp
Last updated: 2026-07-14
Related roadmap: `g10.029`

## Project Problem

Signal completed many bounded mechanism proofs without producing a successor
that improved audible quality. The program repeatedly attributed one smaller
failure and opened another local experiment. Its promoted representation—one
time-adaptive full-band frame per centre—was selected without inspecting the
open source of the comparator it aimed to match.

Rule 30AA's three failed low-tone rows are real, but tracing them further would
only repair a candidate whose broader architecture is no longer supported.

## External Evidence

Signalsmith Stretch demonstrates a coherent single-grid alternative. One long
STFT combines horizontal phase advance with energy-weighted predictions from
both frequency directions and two distances. It does not need hard peak-region
ownership for pure time stretch. Its known `>2x` diffusion fallback keeps it a
control rather than the target.

Rubber Band R3 standard demonstrates the target multi-resolution shape. It
runs long, middle, and short transforms simultaneously but partitions them by
frequency. One full-band classifier supplies H/P/R guidance. It moves
crossovers to spectral valleys and coordinates peak phase, reset, unlock,
attack-energy, and channel-link policy. R3 short removes the multi-resolution
scheme and accepts lower quality for lower cost and delay.

This explains Signal's repeated failure. Signal tested simultaneous redundant
full-band layers, then selected resolution by time. The successful comparator
uses simultaneous non-duplicating frequency ownership with material-guided
phase state.

## Recommendation

Retire the time-adaptive full-band successor. Preserve it only as rejection
evidence.

The next Signal candidate must be one complete source-informed architecture:

1. one synchronized source/output schedule
2. long, middle, and short transforms running at every synthesis step
3. exclusive frequency ownership across those scales
4. one full-band classification reference used only for guidance
5. bounded crossover movement toward local spectral minima
6. explicit ordinary, peak-locked, reset, unlocked, attack, and linked-channel
   phase states
7. per-scale inverse synthesis followed by one sample-aligned sum

The Signal-owned design must derive its policies from stated invariants,
published signal-processing methods, and frozen evidence. Rubber Band source
may identify architecture and failure cases. GPL expression and unexplained
constants do not transfer.

Signalsmith's fixed-grid weighted predictor is the required control. It tests
whether Signal's gain comes from frequency partitioning or simply from replacing
hard phase ownership with multi-direction evidence.

## Rejected Directions

- further Rule 30AB tracing of the retired native-grid candidate
- another window, hop, threshold, or owner-distance sweep
- time-selected full-band resolution
- redundant full-band multi-resolution synthesis
- hard additive H/P/R component stretching as the first target
- direct Rubber Band port or dependency
- Elastique inference beyond public behaviour and API claims

## Required Validation

Use one implementation batch, not a chain of mechanism cards.

- add Signalsmith to the existing frozen synthetic and nine-row development
  comparator set
- implement the complete frequency-partitioned topology behind a report-only
  Signal mode
- retain a single-grid weighted-predictor control using the same schedule and
  output contract
- expose stage ablations in one run: classification guidance, frequency
  partition, reset/unlock/attack policy, and linked-channel phase
- run the complete synthetic gate, all nine mono development rows, and one
  concealed listening pack only if hard integrity passes
- reject or promote the architecture as a whole; do not spawn parameter repair
  batches from individual objective misses

## Promotion

Promoted into:

- `docs/architecture/offline-time-stretch-synthesis.md`
- contract `082`, Rule 31
- roadmap `g10.029`, Batches 29.6CG through 29.6CI

## Outcome

Batch 29.6CI rejects the frequency-partitioned target and retains the weighted
predictor control as the sole successor research direction. The result points
to weighted phase evidence, not simultaneous scale ownership, as the useful
transfer from the source study.

The five-way pack does not establish external ranking: external engines used
full stereo sources while Signal used isolated mono excerpts. Batch 29.6CJ must
repeat the comparator control with exact input identity before the remaining
quality gap can be assessed.

Batch 29.6CJ now provides that exact-input pack. Every path consumes the same
44.1 kHz mono 16-bit `16384`-frame row input; both external engines return the
exact requested length. Listening remains open.

That listening retains weighted prediction as credible research but does not
promote the implementation. Its two wins are offset by repeated softness,
smear, grain, and boundary damage. Batch 29.6CK isolates the remaining unknown:
musical continuity over five-second `1.5x` and `2.0x` renders. A non-win closes
this implementation without tuning.

Long-form listening validates the predictor family: it improves on current
Signal in four of six rows. It does not validate the implementation. One bass
tone mutates, one pad row suffers severe phase damage, and Rubber Band wins four
rows. Source reinspection identifies architectural divergence in transform
duration/interval, time-factor-scaled vertical twists, energy normalization,
weak-evidence fallback, and prediction update order. Those mechanisms now
replace local repair as the next design target.

Memo 005 now freezes the corrected Signal topology. It supersedes the first
control's `2048/128` scheduling and same-frame weighted sum while retaining
weighted prediction as the selected family.

## Sources

| Source | Confidence | Transfer boundary |
| --- | --- | --- |
| [Signalsmith implementation](https://github.com/Signalsmith-Audio/signalsmith-stretch/blob/57b93f4e9206a089a45387eaa39bdc9f310d3308/signalsmith-stretch.h) | high | MIT control architecture; preserve attribution |
| [Signalsmith design](https://signalsmith-audio.co.uk/writing/2023/stretch-design/) | high | predictor rationale and known transient limitation |
| [Rubber Band R3 guide](https://github.com/breakfastquay/rubberband/blob/e4296ac80b1170018a110bc326fd0d45a0eb27d6/src/finer/Guide.h) | high | architecture and state inventory only |
| [Rubber Band R3 phase advance](https://github.com/breakfastquay/rubberband/blob/e4296ac80b1170018a110bc326fd0d45a0eb27d6/src/finer/PhaseAdvance.h) | high | phase-state topology only |
| [Rubber Band R3 stretcher](https://github.com/breakfastquay/rubberband/blob/e4296ac80b1170018a110bc326fd0d45a0eb27d6/src/finer/R3Stretcher.cpp) | high | scale ownership and synthesis topology only |
| [SoundTouch algorithm notes](https://www.surina.net/soundtouch/README.html) | high | WSOLA contrast; not selected |
| [Elastique SDK](https://licensing.zplane.de/uploads/SDK/ELASTIQUE-PRO/V3/manual/elastique_pro_v3_sdk_documentation.pdf) | medium | behavioural claims only; internals unavailable |

## Next Task

Memo 019 does not reopen this rejected prototype. It selects a different
complete kernel in which exclusive scale ownership, synchronized all-channel
phase-state selection, conditional linked trajectories, and per-channel
synthesis are indivisible. Batch 29.7AK passes fixed mechanics and closes at
the sample-rate/duration capacity boundary. Batch 29.7AM validates memo 020's
normalized sliced frame. Run Batch 29.7AN under Rule 31U for guided boundary
mechanics only; do not resume this memo's implementation or Rule 30AB.
