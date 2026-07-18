# g10.029 Direct Scale State Mechanics

Date: 2026-07-18
Batch: 29.7AT
Status: complete; Batch 29.7AU ready

## Result

The private release-only direct scale timeline passes Rule 31Z state mechanics
at hash `430543f8e1dce721`.

- state uses the prepared `2CP` phase and `2CP` region slabs
- processing uses caller-owned current, guidance, output, and state buffers
- reset, attack, scripted ordinary, unlocked, and locked all execute
- ordinary and unlocked recurrence remain channel-local
- compatible borrowing is locked-only and below `6000 Hz`
- exact `6000 Hz` remains local; exact `750 Hz` remains middle-owned
- peer magnitude and current peer peak-relative offset are preserved
- first state and silence recovery reset; exact silence remains zero
- shape failure returns before state mutation
- all proof rates, fixed storage, finiteness, and repeat pass

The dense final locked tick contains `56` compatible borrowed regions and `74`
local regions. The sparse ownership fixture proves one low borrow, one exact-
`6000 Hz` local lock, and owner changes at `1e-12` ownership tolerance.

No representation, crossover, mask, window, schedule, threshold, capacity, or
masked diagnostic changed. No objective or listening audio ran.

## Validation

- focused release state tests: pass, `4/4`
- all direct-scale release tests: pass, `8/8`
- normal `signal-dsp-stretch` suite: pass, `269/269`
- scoped `rustfmt --check`: pass
- release `-D missing-docs` library check: pass
- strict release Clippy reports no direct-scale finding; package-wide Clippy
  remains blocked by `78` existing warnings
- `effigy qa:docs`: pass
- `effigy qa:northstar`: pass
- `effigy health`: pass
- `effigy validate`: pass
- `effigy doctor`: unchanged pre-existing `95` god-file and `5` attention-
  marker findings; this batch adds no finding
- full release suite: `304` pass, `27` ignored, and the same `4` existing
  optimized-build frozen-hash assertions fail outside the direct-scale path
- workspace `cargo fmt --check` remains blocked by unrelated concurrent
  binaural and convolution work; all direct-scale files pass scoped formatting

## Next Task

Run Batch 29.7AU. Freeze the complete failure-first objective sequence before
audio generation, then stop at the first existing hard-gate miss. Keep tuning,
retry, listening, holdout, product work, and Batch 29.8 closed.
