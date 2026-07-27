# 2026-07-27 g10.039 Batch 39.3 First Candidate, Structural Conformance

Status: in structural conformance iteration; three of four gates failed

The first `ResumableOfflineStretch` implementation exists in the isolated
worktree and compiles. It fails the structural gates. Contract `084` Rule 11
permits iteration on compile, construction, and structural conformance before
an acoustic checkpoint is frozen, so the candidate is retained rather than
deleted. No candidate code entered `main`.

## Isolation

The worktree branches from `e24dadc6`, the last commit, which predates the
uncommitted `g10.036` through `g10.038` work. The candidate needs that work as
its base, so the working tree was transplanted as a patch plus untracked files
rather than committing to `main` unasked. The candidate branch is
`candidate/g10-039-resumable`.

## Gate Results

| gate | result |
| --- | --- |
| `G1` chunk-size independence, static ratio | **failed** at chunk `1024` |
| `G1b` chunk-size independence, dynamic ratio | **failed** at chunk `2048` |
| `G2` memory ceiling, duration independence | **failed** at maximum geometry |
| `G3` output length matches target | passed at four ratios |
| `G4` correlation against a whole render | **failed**, `-0.082711` |

## What The Failures Say

`G3` passing and `G1` failing together is the informative pair. Output length is
correct for any partition, so frame *scheduling* is chunk-independent as
designed — analysis frames sit on a grid measured from the source origin. The
sample values still differ, so the defect is in emission and ring management,
not in which frames exist.

`G4` at `-0.082711` is worse than the `0.034` baseline the lane exists to fix.
A candidate that scheduled frames correctly but emitted them wrongly would look
exactly like this, which is consistent with the `G1`/`G3` split.

`G2` measures `11665468` B against the frozen `8388608` B ceiling at maximum
geometry. The cause is a design error in this implementation, not in the
Batch 39.2 brief: the brief sized the overlap-add and normalization rings at
twice the window, and this implementation allocates four times the window for
both those rings and the input ring. The ceiling was derived from the frozen
figure and the code did not honour it.

## Leading Suspects

Not verified, recorded so the next attempt starts from evidence rather than a
fresh guess:

- `drain` calls `emit` mid-loop whenever the ring would overrun, so how much
  output has been flushed depends on when the caller's chunk boundary lands.
  `emit` also clears ring frames as it goes, so an early flush can zero a frame
  a later analysis frame still contributes to
- the output ring is sized like the input ring, but at ratio above `1.0` the
  synthesis cursor advances faster than the analysis cursor, so the output ring
  needs headroom the input ring does not
- `pending_crop_frames` and the target check interleave inside the emit loop, so
  the leading-pad crop can consume a frame that a later call would have emitted

## Next Task

Continue `g10.039` Batch 39.3 under Contract `084` Rule 11: correct emission and
ring sizing against the frozen brief, keep frame scheduling as it is since `G3`
shows it is already partition-independent, and re-run the structural gates. The
acoustic checkpoint opens only once compile, construction, and all structural
gates pass on a clean tree.

If a second attempt fails the same class, Rule 7 applies and the design needs
reassessment rather than another correction.
