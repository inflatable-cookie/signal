# g10.029 Adaptive Single-Frame Phase And Synthesis Proof

Date: 2026-07-13

## Scope

Batch 29.6BR attaches one actual-hop phase state and exact output-lattice dual
synthesis to the frozen study, adaptive ownership, and global map. Event
correction and current-frame vertical locking remain separate modes. Corpus,
holdout, listening, and tuning remain closed.

## Result

The mechanism passes at identity, `0.75`, `1.5`, and `2.0`.

- every control retains `104` selected frames and `24` resolution changes
- phase initializes once per channel and does not reset at resolution changes
- uncovered output and all eight structural failures are zero
- output-frame condition is `1.694641/1.668755/1.863098/2.964471`
- identity peak error is `1.334183e-12`
- `311 Hz` tone error is `0/0.5/0/0 Hz`
- known injected-event error is `128/96/192/256` frames
- symmetry error is zero; maximum imaginary residue is `2.03e-14`
- event and vertical phase modes both change owned phase
- coefficient, magnitude, timing, linked-decision, and repeat hashes are stable
- aggregate evidence hash is `9cc7519deb368966`

The prior identity `6987080e517f1aec`, ownership `2a29d952d91e92ba`, and map
`3ea1d3a2297083e2` evidence remains unchanged.

## Decision

Open Batch 29.6BS under Contract `082`, Rule 30N. This result proves one
non-duplicating synthesis path is live; it does not prove transient or tonal
quality. The `2.0` known-event result sits exactly on the `256`-frame mechanism
limit, so the next gate must use isolated and one-to-one dense-event controls
with search bounds wider than their acceptance limits.

## Next Task

Execute Batch 29.6BS synthetic quality gate without corpus reads or parameter
changes. Keep holdout, listening, tuning, linked-stereo promotion, dynamic
ratio, and product routing closed.
