# g10.031 LayeredCloud Source-Geometry Reconciliation

Date: 2026-07-22
Batch: 31.70 pre-conformance
Result: authority corrected; retained worktree ready to resume

## Stop

Construction stopped before candidate source. The frozen request accepted any
non-empty source, and `S06` required success at `L=1` and `L=H-1`. With
`H=round_half_up(F/64)`, unit-rate reads from the entry and terminal grains
leave zero-validity interior frames when the source is shorter than one launch
hop. That contradicts the required `W(y)>=2^-20` floor.

No candidate DSP, structural render, synthetic render, comparator output, or
acoustic checkpoint existed. The isolated branch remained at the Batch 31.69
docs commit.

## Correction

- require non-empty `L>=H`, about `15.6 ms` at every supported sample rate
- reject shorter input before output allocation
- replace `S06` success lengths `1,H-1` with `2H,12H`
- add the short-source rejection row to `S01`
- freeze structural authority at `101` rows and `51` renders

The map, launch lattice, grain law, normalization, boundaries, stereo,
synthetic, listening, cleanup, and admission surfaces are unchanged. Clamping,
reflection, denser launches, and fill passes remain forbidden.

## Next Task

Apply this commit to `signal-candidate-31-70`, implement the corrected frozen
authority, and pass two complete unchanged conformance rounds before creating
the acoustic ref. Do not create acoustic output during conformance. Keep
`main`, admitted renderers, overlaps, routing, controls, cache, dynamic ratio,
Loophole, and Chorus unchanged. Do not push.
