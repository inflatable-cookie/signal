# g10.031 Geometry-Audited Linked STN Rejection

Date: 2026-07-21
Batch: 31.49
Status: complete; rejected at structural exact-silence admission

## Scope

Implement geometry-audited v4 once in its exact disposable worktree. Freeze
one checkpoint after construction. Run structural and synthetic admission in
order. Stop, delete, and close on the first miss.

## Receipt

- base: `feeb76fe255aa56640de8f732a842942aca936d0`
- checkpoint: `e2ef62f81675b5f31426161644b758097485ce0d`
- tree: `85dc0e455872957fc829439cbb61fcf54d5719a7`
- compile: pass after permitted visibility-only assembly fixes
- construction: `1/1`
- structural: `17/18`; `S01..S14` and `S16..S18` pass
- failed owner: `S15`
- synthetic: not run
- listening: not opened

The canonical brief records every candidate-file, spec, lockfile, toolchain,
and platform digest. No synthetic output exists, so no output digest exists.

## Dominant Cause

Exact-silence input produced deterministic residual samples around `1e-14`.
The residual owner interpolates `ln(power+eps)` when both endpoint powers are
zero, creating tiny positive power after exponentiation. The boundary owner
requires bit-exact zero. The frozen authority is contradictory.

No assertion, epsilon, threshold, DSP, or test was repaired. Structural was
not rerun. Synthetic and listening remained closed.

## Cleanup

Deleted the exact worktree, branch, checkpoint reference, private source,
tests, build state, receipt, and outputs. No candidate DSP, harness, fixture,
API, route, cache, artifact, product, Loophole, or Chorus change entered
`main`. Pre-existing plugin edits remain unstaged and untouched.

## Next Task

Run Batch 31.50 docs-only. Reconcile zero-power residual interpolation with
bit-exact silence across the complete linked-STN owner and every dependent
invariant. Either freeze fresh complete authority under new identity or close
linked STN. Do not repair or recover Batch 31.49 or implement candidate DSP.
