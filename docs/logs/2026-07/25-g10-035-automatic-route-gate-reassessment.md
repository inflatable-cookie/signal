# 2026-07-25 g10.035 Automatic Route Gate Reassessment

Batch 35.4 is complete. Documentation only.

Batch 35.3 froze checkpoint
`50c3d028ae1d5b0d057e74899b84a1a27c0e0038`, tree
`0ff62f572eef222d38ac356d3874c973d78ba2d2`. Normal-profile stretch
regression passed `204/204`. Two unchanged release conformance rounds passed
construction `1/1` and structural `8/8`.

The first acoustic owner stopped on pure Transparent `rademacher-noise` at
`N=96000`, `T=383999=4N-1`. Dispatch and output were byte-exact Transparent.
The owner peak was `10.370356`, above the brief's universal `8.0` ceiling.
Those two requirements cannot both hold. No later synthetic, long-form, or
listening owner ran.

The checkpoint is evidence-invalid, not acoustically passed or rejected. Pure
owner controls now inherit their admitted owner integrity rules. Interior
route rows still reject peak above the larger sample-aligned arm peak by more
than two `f32` ulps. No owner, route, map, weight, source, seed, comparator,
threshold sweep, listening rule, code, or public surface changed.

One exact replay is authorized. Batch 35.5 must restore and hash-prove the
checkpoint source in a fresh worktree, pass conformance twice, freeze a new
acoustic ref, and restart at identity/parity. The Batch 35.3 worktree, branch,
ref, and generated state are deleted after this commit. Public Automatic
remains blocked.
