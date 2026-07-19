# Direct Channel-Local Peak Reassessment

Date: 2026-07-19
Batch: 29.7AY
Status: complete

## Finding

The remaining direct mismatch is upstream of Rule 31AA. Pinned Rubber Band R3
`4.0.0` maintains current and predecessor peak maps per channel. The requesting
channel retains its peak location; a frequency-aligned greatest-channel choice
may lend only a compatible trajectory.

Signal's direct `build_regions` finds peaks and valleys from joint maximum
channel energy. It assigns one peak and one owner to both channel records, and
state processing iterates that shared map. The owner-reference correction can
preserve relation at the chosen shared peak but cannot restore either discarded
channel-local peak identity.

## Falsifier

An analytic two-channel spectrum gives each channel a distinct nearby peak and
compatible predecessor history. The current builder collapses both channel
records onto one peak. A conforming topology must preserve both locations while
allowing frequency-aligned compatible trajectory borrowing. Swap symmetry,
requesting-channel magnitude and local offset, unsupported-peer fallback,
finiteness, fixed capacity, and repeat remain hard invariants.

This requires no corpus audio and no numeric tuning. Frequency/ratio-dependent
offset scaling remains deferred.

## Decision

Keep the direct scale, schedule, state-precedence, recurrence, synthesis, and
capacity architecture. Reject only the joint peak map. Batch 29.7AZ must freeze
the complete channel-local peak mechanics contract before implementation.

## Next Task

Run Batch 29.7AZ under Rule 31AC. Keep state code and objective audio closed.
