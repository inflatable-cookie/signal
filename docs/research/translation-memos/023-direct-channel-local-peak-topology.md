# Direct Channel-Local Peak Topology

Status: promoted for mechanics contract
Date: 2026-07-19
Roadmap: `g10.029`, Batch 29.7AY
Contract: `082`, Rule 31AC

## Question

Which source-backed mechanism remains absent after the Rule 31AA owner-peak
phase-reference correction improves local stereo windows but leaves
`36/48` row-complete failures and the worst residual unchanged?

## Source Finding

Pinned Rubber Band R3 `4.0.0` maintains current and previous peak maps per
channel. For one requesting channel and frequency atom, its own current peak
location remains the region anchor. A greatest-channel decision at that atom
may supply a trajectory only when predecessor peak identities are compatible.
Borrowing changes trajectory ownership, not the requesting peak location.

This is architecture evidence only. GPL expression, peak density, frequency
limits, offset multipliers, reset ranges, and constants remain excluded.

## Signal Mismatch

The direct timeline's `build_regions` finds peaks and valleys from the maximum
energy across channels. `assign_region` then writes the same peak and owner to
every channel record. State processing iterates that one shared region map.

The Rule 31AA correction preserves inter-channel relation at the resulting
shared peak. It does not restore either channel's discarded peak identity.
This explains why the correction is active yet cannot establish full source
conformance. AX aggregate evidence does not identify which rows exercise the
mismatch, so no row-level causal claim is made.

## Selected Mechanism

Retain the direct frequency scales, absolute schedule, channel-local ordinary
and unlocked recurrence, terminal-state precedence, magnitude, per-channel
synthesis, and fixed-storage boundary. Replace only the joint peak topology:

1. current and predecessor peak identities belong to each channel
2. every requesting channel retains its own current peak location
3. possible trajectory ownership is chosen at the requesting frequency atom
4. borrowing requires compatible predecessor peak identity at that frequency
5. a borrowed trajectory is evaluated at the requesting channel's peak index
6. local analysis-relative phase and magnitude remain owned by the requesting
   channel

The exact region iteration, compatibility record, counts, ties, and fixed
storage must be frozen implementation-free before code changes.

## No-Audio Falsifier

Use analytic two-channel spectra within one scale. Give the channels distinct
nearby peak locations, distinct phase relations, and compatible predecessor
histories. The current joint map collapses both records onto one peak. A
conforming map must preserve both peak locations while allowing a compatible
frequency-aligned trajectory owner. It must also preserve swap symmetry,
requesting-channel magnitude and local offset, unsupported-peer fallback,
finiteness, fixed capacity, and exact repeat.

No corpus render is needed to distinguish the topologies.

## Rejected Alternatives

- no parameter sweep or peak-density tuning
- no frequency/ratio-dependent offset multiplier yet
- no shared-peak repair or second phase-reference patch
- no objective rerun before the no-audio topology contract and mechanics pass

## Sources

- [Rubber Band R3 phase advance](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/PhaseAdvance.h)
- [Rubber Band R3 guide](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/Guide.h)
- [Rubber Band linked-stereo mechanism](./007-rubber-band-linked-stereo-mechanism.md)
- [Linked-stereo state and trajectory policy](./008-linked-stereo-state-and-trajectory-policy.md)
- [Direct scale-timeline ownership](./021-direct-scale-timeline-ownership.md)

## Next Task

Run Batch 29.7AZ. Freeze the complete implementation-free mechanics contract
and analytic falsifier before changing the state implementation.
