# 2026-07-25 g10.035 Automatic Route Replay

Batch 35.5 is complete. The route is rejected at synthetic pitch.

The exact replay starts from checkpoint `50c3d028`, applies only the corrected
peak-gate ownership, and freezes checkpoint
`db2a02d35f39a035e44803d0cc26861dcebe2534`, tree
`ab8bf005fe8fe72522e3edc23b617d2ac37b5cd8`.

Compile passes. Two unchanged conformance rounds pass construction `1/1` and
structural `8/8`. Non-acoustic regression passes `204/204`. Corrected
identity/parity passes all `150` rows.

Pitch then rejects low tone at `N=96000`, `T=576000=6N`, `110 Hz`.
Transparent error is `0.16404282837539305` cents, Dream error is
`6.277316077755877` cents, and Automatic error is
`8.717736874188192` cents. Automatic is `2.440420796432315` cents worse
than the worse arm against the frozen `1`-cent allowance.

No later synthetic owner, long-form render, mono listening, or linked-stereo
review runs. No tuning, repair, or replay follows. Nothing enters `main`.

The disposable worktree, branch, generated evidence, and build state are
deleted after this docs commit. The acoustic ref remains through the Batch
35.6 docs-only product and architecture reassessment.
