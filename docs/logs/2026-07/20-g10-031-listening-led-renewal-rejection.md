# g10.031 Listening-Led Renewal Rejection

Date: 2026-07-20
Batch: 31.34
Status: complete; candidate rejected and deleted

## Evidence

- worktree: `signal-candidate-31-34`
- branch: `candidate/g10-031-listening-led-source-relative-renewal`
- immutable checkpoint: `f76d5bb7241cd27f3a897ff9cf1b8c7e678cc91c`
- compile: pass
- construction: exactly `1/1`
- structural: exactly `15/15`
- synthetic: nine selected; eight passed and `Y08` failed
- `Y02`: complete listening-led pitch diagnostic passed
- listening: not run

`Y08` found an exact-zero run of at least one `H` block in the impulse row at
`4x`, `8x`, and `16x`. The frozen test applied the dropout assertion over the
complete impulse output.

The audited prose uses complete impulse output for first-difference crest but
requires dropout absence inside mapped non-zero support. Batch 31.25 also
passed `Y08` under the otherwise matching mono topology. The checkpoint is
still rejected: source, tests, assertions, and manifest froze after
construction, so no assertion repair or rerun is allowed. The receipt does not
yet distinguish renderer-support failure from an over-broad executable support
range.

Cleanup deleted the worktree, branch, checkpoint, module, tests, local build
state, and candidate artifacts. The disposable nextest cache was moved to
Trash. No DSP, harness, fixture, API, route, cache, Loophole, or Chorus surface
entered `main`.

## Next Task

Run Batch 31.35 only. Reconcile the `Y08` complete-impulse measurement range
with mapped non-zero support and the passed Batch 31.25 receipt. Classify the
miss before freezing any new candidate. Do not implement DSP, repair or rerun
the checkpoint, or push.
