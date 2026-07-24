# g10.032 Centred Cyclic Y01 Rejection

Date: 2026-07-24
Status: Batch 32.16 complete; Batch 32.17 ready

## Evidence

Checkpoint `74a6d6d9`, tree `d519e2d8`, ran unchanged against exact frozen
Y01 comparator assets.

- `12` passing receipts: low tone, high tone, chord, and harmonic pad at
  `2x`, `4x`, and `8x`
- first failure: `Y01-012-impulse-r2-c048000`
- error: `unexpected dropout 1`
- failing receipt SHA-256:
  `64eec35d2fef5d7ef3c1d219020d901cff864437469c977680558972c34e7529`
- no Y01 summary
- no Y02 or later execution

The failure means one `221`-frame output window fell below `-80 dBFS` while
its source-mapped window exceeded `-40 dBFS`. This is a valid acoustic
rejection, not evidence plumbing.

## Decision

Reject the centred compressed-anchor checkpoint. Do not repair or rerun it.
Keep the Cyclic product target open. The next batch must attribute the dropout
to complete renderer ownership and decide one materially different architecture
or an evidence-backed stop. Local tuning is prohibited.

## Next Task

Execute Batch 32.17 only. No candidate implementation or acoustic execution.
