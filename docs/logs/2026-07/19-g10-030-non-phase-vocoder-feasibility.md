# g10.030 Non-Phase-Vocoder Feasibility

Date: 2026-07-19
Batch: 30.7 and roadmap closeout
Status: complete

## Study

Assessed complete non-phase-vocoder renderer families against Contract `084`,
the retained comparator and listening evidence, Signal's failed-candidate
history, and primary public research.

- waveform-similarity and source-synchronous overlap-add remain useful
  specialists, but do not jointly own arbitrary polyphony, expansion replicas,
  exact event placement, and linked stereo
- the pinned SBSMS direct-subband sinusoidal topology already failed Signal's
  mono integrity, long-form objective, linked-stereo, and exact mechanics gates
- deterministic sines/transients/noise has useful material separation, but no
  reviewed architecture supplies one timing, recombination, boundary, and
  linked-channel law for every output component
- reviewed learned synthesis targets a different extreme-stretch problem and
  misses Signal's determinism, channel-ownership, training, and first-party
  operating constraints

## Decision

No family has a source-backed reason to clear every whole-renderer gate. No
successor brief or candidate worktree opens. `g10.030` and Contract `084` close
without promotion. The frozen OfflineHighQuality renderer remains production
authority.

This is not a universal rejection of non-phase-vocoder time stretch. Reopening
requires new whole-system evidence: a complete target-ratio renderer that beats
the baseline on comparable material, exposes one linked-channel law, and has a
credible deterministic exact-length bounded-memory design.

## Boundary

- documentation only
- no DSP, harness, fixture, report mode, or experiment module entered `main`
- no Loophole, Chorus, RealtimePreview source-fill, or render-plane work
- production OfflineHighQuality behavior unchanged

## Validation

- `git diff --check`
- `effigy qa:docs`
- `effigy qa:northstar`
- `effigy health`
- `effigy validate`

## Next Task

None in the OfflineHighQuality successor lane. Choose any next `g10` priority
through the generation front door. Keep `g10.028` paused unless separately
authorized.
