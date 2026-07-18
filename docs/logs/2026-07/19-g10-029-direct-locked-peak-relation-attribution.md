# g10.029 Direct Locked-Peak Relation Attribution

Date: 2026-07-19
Batch: 29.7AV
Status: complete, collapse confirmed

## Result

One analytic `48 kHz` state sequence reproduces the AU tone signature without
changing or rerunning the renderer. It executes reset, attack, unlocked, and
locked decisions with one compatible sub-`6000 Hz` borrowed peak and one
exact-`6000 Hz` local peak.

- reset inter-channel relation error: `0`
- attack inter-channel relation error: `0`
- unlocked channel-rotation separation: `0.03333333333333233`
- borrowed input relation: `-0.9500000000000002 radians`
- borrowed output relation: `0`
- borrowed relation loss: `0.9500000000000002 radians`
- exact-`6000 Hz` local-lock rotation separation: `0.1666666666667198`
- borrowed/local regions: `1/1`
- attribution hash: `346e329081adf701`

Reset and attack preserve the input relation. Unlocked and local lock retain
distinct channel rotations. Only compatible borrowing collapses the channel
relation.

## Cause

The borrowed trajectory is shared correctly, but every channel measures its
atom offset from its own current peak. At the peak that offset is zero for all
channels, so all peak phases become the borrowed trajectory phase. Existing
mechanics proved magnitude and within-channel shape but omitted inter-channel
phase at the borrowed peak.

## Frozen Correction

For compatible borrowing only, measure each atom's current analysis-relative
phase from the current owner peak. Local locking continues to reference the
same channel's peak. This is equivalent to using `trajectory_channel` for both
ordinary recurrence and the current peak-phase reference.

No other state, region, magnitude, geometry, mask, schedule, threshold,
capacity, or synthesis field may move.

## Boundary

No production state code, renderer, candidate audio, objective corpus,
listening artifact, concealed material, or holdout material changed or ran.

## Next Task

Run Batch 29.7AW under Rule 31AA. Apply the single frozen reference correction
and prove complete mechanics without corpus audio.
