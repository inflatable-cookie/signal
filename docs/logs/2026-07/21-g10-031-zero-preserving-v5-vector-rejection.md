# g10.031 Zero-Preserving V5 Vector Rejection

Date: 2026-07-21
Batch: 31.51
Status: complete; rejected at structural geometry-vector admission

## Scope

Implement zero-preserving v5 once in its exact disposable worktree. Freeze one
checkpoint after construction. Run structural and synthetic admission in
order. Stop, delete, and close on the first miss.

## Receipt

- base: `570da1604ba21204c1dccfb3aed6d2980ed239ac`
- checkpoint: `959094513b6847cdeb8a3c0bf424efd09ce1fb6f`
- tree: `080bea3698e5b70760edd6b38dcccb995697d2c2`
- compile: pass without repair
- construction: `1/1`
- structural: `17/18`; `S01` and `S03..S18` pass
- failed owner: `S02`
- synthetic: not run
- listening: not opened

The canonical brief records every candidate-file, spec, lockfile, toolchain,
and platform digest. No synthetic output exists, so no output digest exists.

## Dominant Cause

The checkpoint's handwritten 8 kHz geometry vector asserted `Q_h=5`. The
frozen formula and renderer both produce:

`odd(round(0.240*8000/256))=odd(8)=9`.

Construction checked exhaustive maxima but did not independently cross-check
the exact per-rate structural vector. This is executable-evidence failure, not
a renderer-geometry result.

No formula, vector, assertion, DSP, or test was repaired. Structural was not
rerun. Synthetic and listening remained closed.

## Cleanup

Deleted the exact worktree, branch, checkpoint reference, private source,
tests, build state, receipt, and outputs. No candidate DSP, harness, fixture,
API, route, cache, artifact, product, Loophole, or Chorus change entered
`main`. Pre-existing plugin edits remain unstaged and untouched.

## Next Task

Run Batch 31.52 docs-only. Independently audit every exact geometry vector,
witness, rounding tie, and construction assertion. Bind the structural table
into construction authority before deciding whether fresh identity exists.
Do not repair or recover Batch 31.51 or implement candidate DSP.
