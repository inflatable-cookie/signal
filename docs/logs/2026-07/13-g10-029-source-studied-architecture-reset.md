# g10.029 Source-Studied Architecture Reset

Date: 2026-07-13
Batch: 29.6CG
Result: complete

## Decision

Stop repairing the Rule 30 time-adaptive full-band successor. Its three failed
`55 Hz` rows remain valid rejection evidence, but another local attribution
would optimize a representation unsupported by the comparator architecture.

Batch 29.6CH will test one complete frequency-partitioned multi-resolution
architecture against one fixed-grid weighted multi-predictor control. It will
not open a parameter lattice or another per-metric repair chain.

## Source Boundary

- Signalsmith Stretch revision
  `57b93f4e9206a089a45387eaa39bdc9f310d3308`, MIT
- Rubber Band revision
  `e4296ac80b1170018a110bc326fd0d45a0eb27d6`, GPL-2-or-later/commercial

The study transfers architecture, invariants, state ownership, and failure
handling only. Rubber Band source expression, copied control flow, and
unexplained constants do not transfer into Signal. Signalsmith code also
requires attribution and a deliberate Signal-owned boundary if any code is
later reused. No external source was copied in this batch.

## Findings

### Signalsmith Stretch

- one fixed STFT for pure time stretch
- local input/output block lengths define the time map
- horizontal phase prediction followed by immediate and longer vertical
  predictions from both frequency directions
- prediction mixtures weighted by observation energy
- highest-energy channel phase ownership with relative channel phase retained
- deliberate phase diffusion above large stretch ratios as an acknowledged
  smear-versus-alias tradeoff

This is the fixed-grid control, not the target architecture.

### Rubber Band R2

- one phase-vocoder scale
- magnitude transient study separated from synthesis
- signed local increment schedule
- reset, horizontal advance, descending-bin lamination, and synthesis as
  separate stages
- vertical coherence inherited conditionally from higher-frequency neighbours

### Rubber Band R3

- standard mode runs classification, long, and short scales simultaneously
- long, classification, and short scales own low, middle, and high frequencies
  respectively; they are not full-band alternatives selected over time
- crossovers move toward bounded local spectral valleys
- full-band H/P/R classification guides control but is not additive H/P/R
  synthesis
- guidance controls low-frequency kick energy, phase reset, high-frequency
  unlock, silence/unity handling, and channel lock
- each scale performs ordinary instantaneous-frequency advance before
  reset/unlocked/peak-locked treatment
- synthesis zeros bins outside each scale's frequency interval, inverse
  transforms each scale, and sums the aligned scale accumulators
- high-frequency unlocking and channel decoupling increase at large stretch
  ratios

## Correction

Signal adapted resolution on the wrong axis. Its promoted candidate chose one
full-band resolution at each time centre. Rubber Band R3 standard keeps the
scales simultaneous and assigns each output frequency to one scale. Signal's
older redundant full-band union was also wrong because every scale synthesized
the same frequency range.

The target is therefore simultaneous, non-duplicating frequency ownership with
classification-guided phase state. The H/P/R labels remain a decision surface;
they do not become separately stretched additive components.

## Closed Directions

- Rule 30AB low-frequency projection repair
- time-selected full-band resolution
- redundant full-band multi-resolution synthesis
- additive H/P/R synthesis as the first source-studied target
- threshold, window, crossover, or phase-distance sweeps
- direct Rubber Band port or dependency

## Next Task

Execute Batch 29.6CH under Rule 31. Build the report-only
frequency-partitioned long/middle/short candidate and fixed-grid weighted
multi-predictor control, then run the frozen synthetic and nine-row mono
development comparison as one architecture decision.
