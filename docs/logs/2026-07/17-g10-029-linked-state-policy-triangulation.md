# g10.029 Linked-State Policy Triangulation

Date: 2026-07-17
Batch: 29.7N
Status: complete

## Scope

Separate 29.7M peak sharing from its channel-independent fallback, then derive
the narrowest license-safe linked-state policy supported by source architecture,
published work, and Signal's frozen calibration matrix.

## Evidence

The repeat-stable `48`-row ablation compares current reference-relative Signal,
fully channel-independent recurrence, and independent recurrence plus 29.7M
sharing. Failures are `20`, `40`, and `29`. Peak sharing improves all `24` tone
rows relative to independent recurrence, but regresses `22/24` image rows.
There are zero structural failures. Evidence is `d2de8ca4df6330f6`.

## Decision

The dominant 29.7M loss is channel-independent fallback. Its second fault is
using one dominant peer peak as both channels' anchor. Current
reference-relative recurrence remains the default. Later state ownership is
ordered `Reset`, `TrackedPeak`, then `Relational`. A tracked overlay keeps each
channel's peak location and advances from matched predecessor synthesis state.
Independent `Unlocked`, kick-specific laws, parameter tuning, and production
use remain closed.

## Next Task

Run Batch 29.7O. Test one report-only reference-safe tracked identity overlay
without phase scaling, peak-resolution changes, predecessor-distance tuning,
frequency-range tuning, or reset implementation.
