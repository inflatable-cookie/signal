# g10.031 Source-Relative Vector Rejection

Date: 2026-07-20
Batch: 31.27
Status: complete; candidate rejected and deleted

## Evidence

- worktree: `signal-candidate-31-27`
- branch: `candidate/g10-031-source-relative-renewal`
- immutable checkpoint: `1f05cc33dcc57b5714f02bf71f05a44d4ff98f09`
- compile: pass
- construction manifest: exactly `1/1` passed
- structural admission: exactly `15` selected; `14` passed, `S04` failed
- synthetic, mono-listening, and stereo gates: not run

`S04` compared `mix64(1)` with a frozen hexadecimal vector. The implementation
of the normative wrapping expression returned `0x5692161d100b05e5`. The
assertion expected `0x569216d1009b05e5`; it transposed the middle `1d10` into
`d100`.

The checkpoint is rejected as an evidence-construction failure. This result
says nothing about synthetic quality, mono listening, or the source-relative
stereo law. The assertion was not repaired and structural admission was not
rerun.

Cleanup removed the worktree, branch, checkpoint, private module, tests, build
state, and candidate artifacts. No DSP, harness, fixture, API, route, cache,
Loophole, or Chorus surface entered `main`.

## Next Task

Run Batch 31.28 only. Reconcile executable vector ownership, audit every exact
construction vector, and either freeze fresh complete candidate authority
under a new identity or close the topology. Do not implement candidate DSP in
the same batch.
