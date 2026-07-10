# g10.029 Tail Listening Pack

Date: 2026-07-10
Status: complete; unconditional zero anchor rejected

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

## Operator Result

Notes were frozen for all six trials before candidate identity was revealed.

- `T001`: zero anchor produced a low-end thump; current was smoothest
- `T002`: same result as `T001`
- `T003`: current clicked, source anchor clicked less, zero anchor was cleanest
- `T004`: current had a very slight low thump; no strong preference
- `T005`: no material difference
- `T006`: no material difference

The two clearest sustained-pad cases reject the zero control. The clearest drum
case supports it. Objective exterior-step improvement therefore does not predict
local sound across material classes.

## Decision

Reject unconditional promotion of the additive zero-tail anchor. Keep current
production DSP and cache identity unchanged. The source anchor remains rejected.

The material split points at the correction law, not just its size. The current
zero control adds a half-cosine offset reaching the negative final-sample value.
On sustained material that offset can appear as new low-frequency movement. A
multiplicative fade can reach zero over the same span without injecting that
additive offset.

Mono notes resolve this candidate only. Linked-stereo evidence and independent
stereo listening remain promotion blockers for any successor.

## Next Task

Add one report-only 256-frame multiplicative half-cosine terminal fade. Compare
it against current and the rejected additive zero anchor through the 60-row
objective gate, then regenerate the same six concealed tail trials if it passes.
