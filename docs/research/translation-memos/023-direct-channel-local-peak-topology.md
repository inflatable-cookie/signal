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

## Frozen Mechanics

Batch 29.7AZ keeps the existing shared material guidance and one terminal-state
decision per atom. Peak maps, valleys, fallback maxima, and predecessor maps
become channel-local within each scale. The possible trajectory channel is the
greatest channel at the requesting atom; an exact tie selects the lower
channel. Borrowing is below `6000 Hz`, requires equal predecessor peak identity
and supported owner history, and never replaces the requesting peak index.

The selected channel's ordinary advance at the requesting peak is re-anchored
to the common predecessor's prior synthesis phase. The requesting atom then
keeps its current analysis-relative offset from the selected channel at that
requesting peak and keeps its own magnitude. This makes predecessor identity
part of trajectory construction rather than an eligibility flag only.

Current/predecessor region storage remains `2CP`; phase storage remains `2CP`;
terminal state remains `P`. Reports count locked channel-atoms and channel-peak
disagreements. No duration-growing storage or new parameter exists.

## No-Audio Falsifier

Use analytic two-channel spectra within one scale. Give the channels distinct
nearby peak locations, distinct phase relations, and compatible predecessor
histories. The current joint map collapses both records onto one peak. A
conforming map must preserve both peak locations while allowing a compatible
frequency-aligned trajectory owner. It must also preserve swap symmetry,
requesting-channel magnitude and local offset, unsupported-peer fallback,
finiteness, fixed capacity, and exact repeat.

No corpus render is needed to distinguish the topologies.

Batch 29.7BA implements the promoted boundary. The former joint builder maps
the staggered fixture to peak `11`; corrected channel records retain peaks `9`
and `11`, borrow a compatible predecessor-anchored trajectory without moving
the requesting peak, and repeat at `fcbdfd991bd04db1`. The full fallback,
tie, boundary, swap, recovery, terminal, proof-rate, storage, shape, finite,
and repeat matrix passes. No objective audio ran.

Batch 29.7BB freezes the only permitted objective candidate. Direct
channel-atom diagnostics retain their own semantics; they are not translated
into the source-studied harness's shared-region or reference fields. Candidate
`signal-direct-channel-local-peak-v1` must retain every BA mechanics receipt
before the unchanged AX evidence order runs once.

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

Run Batch 29.7BC under Rule 31AD. Apply only the private diagnostic correction,
then execute the frozen objective order once through its first miss.
