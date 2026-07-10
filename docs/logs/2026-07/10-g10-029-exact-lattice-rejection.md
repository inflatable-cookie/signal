# g10.029 Exact-Lattice Rejection

Date: 2026-07-10
Status: rejected
Batch: `29.6H`

Absolute analysis centres stayed within `0.4` frame of the ideal map on all
`60` rows. Intervals were monotonic floor/ceiling steps; assignments, heap,
symmetry, coverage, length, and finite-output gates passed.

The complete mono gate failed: `L001` improvement `2.379387 dB`; worst crest
`4.739765 dB`; mean/worst timing regression `17.789744/151.25` frames;
integrity `57/60`; replica `27/48` with worst `+0.726570`; transient `16/60`;
tonal `57/60`; formant `11/60`; boundary `51/60`; combined `3/60`.

Expansion fast movement, residual, and unsupported-bin means remained improved.
Exact lattice removed a real confound but did not close the dominant placement
and shape defects. Reject without tuning. Linked stereo remains closed.

Local evidence: `target/stretch-corpus-g10-029-exact-lattice-phase-gradient-v1.tsv`.
