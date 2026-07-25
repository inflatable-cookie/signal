# g10.035 Automatic Route Brief

Status: Batch 35.2 complete; Batch 35.3 ready

Batch 35.2 freezes one complete private
`ExactTargetTransparentDreamRouter`:

- exact integer domain `ceil(N/2)..=16N`
- direction-specific promoted Transparent selectors
- one linear source/output map and exact `[0,T)` lattice
- byte-exact Transparent at `4N`
- one render-wide log-ratio smoothstep over `4N<T<8N`
- one convex linear-amplitude blend with no adaptive gain
- byte-exact neutral Dream at `8N`
- linked decisions, deterministic identity, exact boundaries, and two-buffer
  staging
- fixed conformance, synthetic, long-form mono, and linked-stereo gates
- terminal rejection, cleanup, and minimal private admission

The canonical authority is
`docs/architecture/offline-automatic-exact-target-transparent-dream-router-brief.md`.

No DSP, candidate harness, fixture, report mode, public API, cache, artifact,
runtime, UI, Loophole, or Chorus surface changed. No candidate worktree,
branch, or evidence state was created.

Validation passed: `git diff --check`, `effigy qa:docs`,
`effigy qa:northstar`, `effigy health`, and `effigy validate`. `effigy doctor`
reports only the pre-existing `59` god-file and `5` attention-marker findings.

## Next Task

Execute `g10.035` Batch 35.3 only. Create the named disposable worktree,
implement the frozen private route, pass conformance twice, freeze one acoustic
checkpoint, and follow the brief's terminal evidence order.
