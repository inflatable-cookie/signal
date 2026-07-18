# Corrected Direct Objective Preregistration

Date: 2026-07-19
Batch: 29.7AX
Status: active, sequence frozen before audio

## Immutable Inputs

Batch 29.7AX reruns the AU objective sequence after only the Rule 31AA borrowed-
owner peak reference correction. Direct geometry, masks, windows, absolute
source projection, state order, thresholds, ties, capacities, corpus rows, and
renderer fields do not move.

Required no-audio receipts are:

- Rule 31Z representation: `fdf90f6127749341`
- corrected direct state: `52d6b8b2bb6edff0`
- corrected borrowed relation: `425400ebb580b3e1`

## Failure-First Order

1. Rerun all release direct representation, state, and Rule 31AA relation
   mechanics.
2. Run silence, tone, noise, impulse, mixed, and transient sources at `0.75`,
   `1.5`, and `2.0`. Require exact crop and coverage, zero structural and
   nonfinite failures, deterministic repeat, all non-scripted terminal states,
   every hard channel-mechanics error at or below `1e-6`, exactly `19` guidance
   ticks, and every fixed storage cap.
3. After synthetic passage, run the corrected `48`-row stereo corpus once.
   Require zero calibrated and structural failures, deterministic repeat, at
   least `245/384` improved local windows, at most `13/48` Signal-relative
   local-row failures, and maximum normalized-Gram residual at or below
   `0.01744693815260`.
4. After stereo passage, run the unchanged six exact-source mono rows. Require
   zero hard failures and no row-complete regression against current Signal.
5. After mono passage, run their unchanged long-development measurements with
   the same zero-failure and no-row-regression rule.

## Stop Rule

Stop at the first miss. Do not sweep, repair, retry, tune, fall back, listen,
export audio, inspect concealed material, read holdout material, or change a
threshold. Only complete passage may open Batch 29.8.

## Next Task

Execute this sequence through the first hard miss.
