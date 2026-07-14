# Source-Studied Stretch Architecture

Status: partially validated; weighted predictor retained
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

Run the exact-input Batch 29.6CJ comparator confirmation. Do not resume the
frequency-partitioned path, Rule 30AB, or per-mechanism repair batches.
