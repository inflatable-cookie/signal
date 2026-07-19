# g10.031 Cyclic Owner Closure

Date: 2026-07-19
Status: Batch 31.15 complete; explicit cyclic closed

## Decision

Close explicit `Cyclic` without promotion. No third materially different,
source-backed whole-renderer path owns the retained `2x`, `4x`, and `8x`
pitch, scheduled-replica, exact-length, bounded-state, linked-stereo, and
musical gates together.

## Evidence

- `CyclicGrain` fixed source-offset unit-rate reads and failed the first
  synthetic pitch row by `5.778` cents.
- `SimilarityAlignedCyclic` changed segment ownership to bounded waveform
  similarity, then failed structural known-offset recovery because its coarse
  shortlist hid an exact between-grid continuation from full refinement.
- SOLA, WSOLA, SoundTouch-style search, exhaustive search, denser coarse
  sampling, larger shortlists, and score changes remain the rejected alignment
  family or direct repairs to it.
- TD-PSOLA and ESOLA depend on pitch- or epoch-synchronous speech ownership.
  ESOLA's published high-quality exact-scaling claim covers speech from `0.5x`
  through `2x`, not mixed-program `8x`; no reviewed method supplies one
  channel-shared full-mix epoch owner.
- FESOLA adds correlation to epoch alignment and reports voice or
  strong-fundamental solo-instrument use. It combines the same period and
  similarity owners rather than closing the retained full-mix boundary.
- fixed grains repeat the first failed topology; transient/component hybrids
  reopen event-timing or recombination seams; spectral, sinusoidal, and learned
  hybrids reopen closed owners or require a separate product research program.

Primary evidence retained in the canonical study:

- Verhelst and Roelands WSOLA
- official SoundTouch algorithm notes and studied source revision
- Moulines and Charpentier PSOLA
- Rudresh et al. ESOLA
- Roberts and Paliwal FESOLA
- pinned Potenza Akai-style source

## Boundary

Changed documentation only. No candidate, harness, fixture, comparator audio,
report mode, public API, dependency, cache, route, generated audio, Loophole,
or Chorus surface changed. The three unrelated binaural/reverb edits remain
untouched.

`Cyclic` remains useful comparator and future intent vocabulary, not an
available character. `Dream`, `Spectral`, `Rough`, `Cloud`, automatic routing,
dynamic ratio, cache, and product integration remain paused. `g10.031` has no
ready implementation batch.

## Validation

- `git diff --check`: passed
- `effigy qa:docs`: passed
- `effigy qa:northstar`: passed
- `effigy health`: passed
- `effigy validate`: passed
- `effigy doctor`: unchanged pre-existing god-file and attention-marker
  findings

## Next Task

No autonomous creative-stretch task is ready. Reopen `g10.031` only from new
complete-system owner evidence or an explicit operator decision to start a
separate creative research program.
