# Channel-Local Peak Objective Gate

Date: 2026-07-19
Batch: 29.7BC
Status: rejected at stereo gate

## Report Correction

Candidate `signal-direct-channel-local-peak-v1` now reports borrowed/local
locked channel-atoms, trajectory-channel switches, and channel-peak
disagreements in direct units. Foreign source-studied adapter diagnostics are
zero. Reports live under `target/stretch-direct-channel-local-peak-v1`; AX
evidence is unchanged.

## Entry And Synthetic Gates

Release mechanics pass `12` tests with `2` objective tests intentionally
ignored. Receipts remain `fdf90f6127749341`, `5ae654162d4ed279`,
`2b8104525bad0418`, and `fcbdfd991bd04db1`.

Synthetic evidence passes at `ce696ab8cb37b17f`: zero structural, nonfinite,
or mechanics failures; exact repeat; required state coverage; `11322` borrowed
and `54388` local locked channel-atoms; `9772` trajectory-channel switches;
zero channel-peak disagreements; and fixed `10/19/7680` high-water.

## Stereo Stop

The one `48`-row invocation rejects at `b13c37cff1b58afa` with `40/48`
calibrated failures, `159/384` improved windows, `36/48` Signal-relative local
failures, maximum normalized-Gram residual `0.7611955347641768`, zero
structural failures, and exact repeat.

AX recorded `38/48`, `157/384`, `36/48`, and the same worst residual. The new
mechanism gains two windows but loses two calibrated rows and does not move the
dominant local or residual signature. Mono and long-development do not run. No
retry, tuning, repair, listening, export, holdout access, or product change
occurred.

## Next Task

Run Batch 29.7BD under Rule 31AE. Compare retained AX/BC rows and audit direct
phase/synthesis code without new audio. Name one mechanism with causal reach
over the unchanged hard signature or close the direct peak-ownership topology.
