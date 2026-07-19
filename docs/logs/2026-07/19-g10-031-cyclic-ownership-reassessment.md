# g10.031 Cyclic Ownership Reassessment

Date: 2026-07-19
Status: Batch 31.12 complete; new brief selected

## Decision

Select one materially different family for a complete docs-only brief:
`SimilarityAlignedCyclic`.

Rejected `CyclicGrain` fixed every source anchor to the ideal map, then
crossfaded source-offset unit-rate reads. Its first synthetic row failed pitch.
The selected family instead chooses each new source segment by deterministic
waveform similarity to the prior segment's natural continuation inside a
bounded ideal-map neighbourhood.

The future brief must freeze one regular output lattice, one strictly
increasing selected source path, non-accumulating ideal-map displacement, one
channel-symmetric score and offset, native-channel synthesis, exact length,
bounded state, semantic macros, and the retained admission sequence.

## Evidence And Boundary

The original WSOLA work supports correlation-selected segment placement as a
join-continuity mechanism. Maintained SoundTouch architecture independently
supports one overlap search, one selected offset, and shared multichannel
processing. Signal studied SoundTouch revision
`f738b1132ec1fd56efc90367898244cf52d9e6a5` for architecture only.

The earlier transparent WSOLA closure remains intact. A single waveform lag
can compromise polyphony; long sequences and wide searches can produce echo
or drift. Those are candidate risks, not waived defects. Intentional cyclic
repetition is the character target, but frozen pitch, integrity, mono,
`16x` rejection, linked mechanics, and independent stereo gates still apply.

Pitch-synchronous OLA lacks a full-mix period owner. Fixed repetition and
another unaligned grain lattice retain the failed join. Spectral correction
would reopen closed families. None is selected.

No DSP, candidate module, harness, fixture, comparator audio, report mode,
public API, cache, route, dependency, Loophole, or Chorus surface changed.
The three unrelated binaural/reverb edits remain untouched.

## Next Task

Execute `g10.031` Batch 31.13 only. Freeze one complete clean-room
`SimilarityAlignedCyclic` implementation brief. Do not implement it in the
same batch.
