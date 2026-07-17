# g10.029 Reference-Safe Tracked-Peak Overlay

Date: 2026-07-17
Batch: 29.7O
Status: complete

## Scope

Test one report-only tracked identity overlay over the frozen relational
renderer. Keep each channel's current peak location, use the frozen eligibility
rule, advance from matched predecessor synthesis phase, and retain identity
analysis-relative offsets.

## Evidence

The candidate changes `546669` eligible bins and leaves `1304723` on the
relational baseline. Structure, mechanics, silent-peer safety, and repeat pass.
Calibrated failures rise from `20/48` to `25/48`. No row improves completely,
all `48` regress on at least one metric, and `34/48` fail local consistency.
Evidence is `ec1f63ad4bae9fc8`.

## Decision

Reject without parameter rescue. Conventional tracked-peak phase propagation
is unsafe when applied after Signal's broad phase-gradient field is already
integrated. The causal operator conflict remains an attribution target. Current
relational output stays frozen.

## Next Task

Run Batch 29.7P. Attribute peak-owner and phase-field integration order from
primary literature, permissive implementations, and the frozen failure matrix
before authorizing another renderer.
