# g10.029 Phase-Lock Control Rejection

Status: evidence complete; no production candidate
Date: 2026-07-10

## Question

Does broad identity phase locking or overlap-add reconstruction cause the
visible `L001-B` transient spike, and can an existing bounded phase-lock variant
remove it without moving the defect?

## Same-Event Result

The control anchor is Signal Default's worst `L001` source event at frame
`180354`, projected into the `0.75x` render:

- Signal Default identity locking: `5.655 dB` crest growth
- Rubber Band: `-0.515 dB`
- Signal independent bins: `0.459 dB`
- stability-adaptive locking: `0.183 dB`
- tracked peak regions: `5.485 dB`
- magnitude slew: `5.779 dB`

Independent bins use the same STFT window and overlap-add reconstruction as
Signal Default but do not reproduce the spike at that event. Overlap-add is not
the direct cause. Broad identity locking is.

## Full-Pack Gate

The 60-render pack produced 48 rows with a current Signal transient anchor.
Every control reports both the same source event and its own worst event so a
candidate cannot pass by moving the spike.

Existing variants:

- stability-adaptive locking fixed `L001`, but worsened mean event timing in
  `37` rows and improved it in `10`; worst crest improved in `24`, regressed in
  `23`, and was inconclusive in `1`
- tracked peak regions regressed worst crest in `28` rows versus `20`
  improvements and worsened timing in `29` versus `14` improvements
- magnitude slew regressed worst crest in `34` rows versus `12` improvements;
  two rows were inconclusive

Two tighter report-only probes were tested and removed:

- bounding every unstable-frame peak region reduced the mean anchored crest
  from `1.337 dB` to `0.628 dB`, but worsened timing in `35` rows and improved it
  in `12`
- bounding only strong compression-transient frames reduced `L001` to
  `3.524 dB`; on the 16 compression rows it still worsened timing in `9` and
  improved it in `7`, while worst crest regressed in `7`

Neither probe was retained. No production DSP path changed.

Target-local evidence:

- `target/stretch-corpus-g10-029-phase-lock-control-final-v1.tsv`
- `target/stretch-corpus-g10-029-phase-lock-control-v1.tsv`
- `target/stretch-corpus-g10-029-phase-lock-control-v2.tsv`
- `target/stretch-corpus-g10-029-phase-lock-control-v3.tsv`

## Decision

Reject local phase-lock heuristics as the fix for this quality gate. They can
trade crest shape against measured event placement but do not improve both
across the corpus.

The next structural design must separate transient and tonal ownership and
provide an explicit transition mechanism. Do not promote a phase-lock selector
from this evidence.

## Next Task

Measure the reported long-stretch grain and atonal ringing with bounded tonal
sideband, residual, and modulation evidence. Keep the structural hybrid and
product promotion closed until independent stereo review is complete.
