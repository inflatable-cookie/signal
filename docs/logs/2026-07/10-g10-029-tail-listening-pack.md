# g10.029 Tail Listening Pack

Date: 2026-07-10
Status: ready for operator notes

## Purpose

Test whether the zero-tail control replaces the current exterior click with an
audible pull, thump, or damaged tail. Keep candidate identity concealed until
notes are frozen.

## Selection

The exporter re-rendered all 60 broad-manifest rows and ranked the absolute
current final sample. This is both the jump into digital silence and the zero
control's maximum correction. The six selected trials cover corrections from
`0.482575566` to `0.229887158`, equivalent to current exterior steps from
`-6.328693` to `-12.769706 dBFS`.

Repeated sources and ratios remain when they are true worst cases. No diversity
substitution displaced a larger correction.

## Pack

Target-local path:
`target/stretch-corpus-g10-029-tail-listening-pack-v1`

- six trials
- three candidates per trial: current, rejected source anchor, qualified zero
  anchor
- deterministic A/B/C permutation with identity only in
  `tail-listening-key.tsv`
- mono final-second excerpts
- `250 ms` digital silence after the exact render endpoint
- one shared per-trial gain, targeting `0.15` RMS with a `0.95` peak ceiling
- click/pop, pull/thump, continuity, preference, notes, and completion fields
  in `tail-listening-notes.tsv`

The shared gain uses the current excerpt RMS and the peak across all three
candidates. It preserves relative endpoint amplitude and cannot normalize away
the boundary under test.

## Current State

The pack is ready. No listening result is inferred from objective metrics and
no note row is marked complete. Production DSP and cache identity remain
unchanged. Mono notes can qualify local tail sound only; linked-stereo evidence
and independent stereo listening remain promotion blockers.

## Next Task

Complete all six concealed trials. Freeze notes before opening the key. Classify
whether zero anchoring removes click/pop without adding pull/thump or damaging
tail continuity.
