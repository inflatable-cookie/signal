# Linked-Stereo Mechanics

Date: 2026-07-16
Roadmap: `g10.029`, Batch 29.7B
Scope: report-only coherent predictor stereo mechanics

## Decision

Pass mechanics. Open Batch 29.7C quality controls without changing the shared
decision or per-channel recurrence.

## Evidence

- ratios: `0.75x`, `1.5x`, `2.0x`
- geometry: `[960, 240, 1024, 512]`
- identity mismatches: `0`
- structural failures: `[0; 4]` on every ratio
- duplicated-mono mismatches: `0`
- hard-pan active-channel mismatches: `0`
- silent-channel peak: `0`
- channel-swap mismatches: `0`
- polarity mismatches: `0`
- scaled-duplicate mono-parity mismatches: `0`
- non-silent unilateral completions: `0`
- shared corrected and fallback counts: non-zero on every ratio
- repeated review: exact

Frozen row audio hashes:

- `0.75x`: `38dad9d73677280f`
- `1.5x`: `a48d55bf5f1120ae`
- `2.0x`: `d90c4971bd452d50`
- aggregate audio: `f34476f290ce4f80`
- evidence: `426af565378e9ce1`

## Contract Correction

The first contract draft required inverse-normalized gain equivariance. That is
not a valid invariant of the frozen mono topology because horizontal prediction
uses a fixed absolute energy floor. The stereo invariant is bit-exact parity
with the mono renderer at each declared gain. Duplicate controls at gains
`0.25` and `4` pass.

One first-run bin also exposed a unilateral non-silent completion. The final
shared mode now falls back for both channels when either significant channel's
prediction is individually degenerate. No threshold changed.

## Next Task

Implement Batch 29.7C constant-IPD, broadband-delay, mid/side-ratio,
correlation, and one-sided-transient controls. Stop on the first frozen gate
failure; do not tune the passing mechanics.
