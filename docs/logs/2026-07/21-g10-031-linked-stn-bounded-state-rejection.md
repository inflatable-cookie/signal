# g10.031 Linked STN Bounded-State Rejection

Date: 2026-07-21
Batch: 31.43
Status: complete; candidate rejected and deleted

## Scope

Implement the frozen `LinkedStnNoiseMorph` brief once in its named disposable
worktree. Run compile and construction, freeze one checkpoint, then run the
structural and synthetic gates in order. Stop and clean up on any miss.

## Baseline And Isolation

- main start: `c84bd5383f8876d47050c42e00f87f4cf233cf22`
- worktree: `/Users/tom/Dev/projects/signal-candidate-31-43`
- branch: `candidate/g10-031-linked-stn-noise-morph`
- checkpoint: `1c38367987290fcca6743808a0d6dcc7f28d564c`
- tree: `cf413de5f9b51af181244facc30c28f7d32c6b11`
- candidate delta: nine files, `2722` insertions
- public API, production DSP, dependencies, routes, cache, artifacts,
  Loophole, and Chorus: unchanged

The candidate contained the eight required private module files plus one
private `lib.rs` declaration. No candidate source entered `main`.

## Executable Identity

SHA-256:

| Surface | Digest |
| --- | --- |
| `lib.rs` candidate declaration | `f0dcd1fa44c720345a8e1c43cb879f4398636bac242c319bf0899aac419ff5f0` |
| `decomposition.rs` | `4120be90f7e7f8992ba3e6a1d66dc44afa4bcb7366dd2dd739f04c823cbb4eed` |
| `mod.rs` | `bbe848c6b5be5e83a35c5bdb681ef6654ce05dea3ef5f4620ebc39a5e54ffe69` |
| `noise.rs` | `7962b350c8646019e053aa29f8ec1d362544757fdf9bcfa0202263c2ef8ac718` |
| `plan.rs` | `c4122d443d9e43c03e7abaa0afa045dc22b94fb7c353b0ac824fbbc7577f3559` |
| `synthesis.rs` | `e13b1e59712b5f7257707d293cbba9d9b75d9fd518d2c187f29bddd7674d3764` |
| `tests.rs` | `992a7b58c5180f56f834d89505f0fb92718b6c0ae13ea0e3ce39c4bb1fcd3904` |
| `tonal.rs` | `36847f096e9885fa72822404f62d93e0e01ab33f7b0b760c3ab86a92fb148e39` |
| `transient.rs` | `68d6c54eb9a7af14b986199c7a8f1e0a0bce5f4d09dfef4ddd0646801bf27f4c` |
| exact `EVIDENCE_SPEC` declaration | `6875aad9cf754b2249a8677b009f41b81f2c66ece7daae2ea0e6edfc230ca166` |
| `Cargo.lock` | `e3848a40d2ea1ff88a0e036df40d1fefa56c7aca950a95262c1d8c5668fd394d` |

Toolchain and platform:

- `rustc 1.96.0 (ac68faa20 2026-05-25)`
- rustc commit `ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`
- host `aarch64-apple-darwin`; LLVM `22.1.2`
- macOS `26.5.2` build `25F84`
- Darwin `25.5.0`, arm64

## Gate Receipt

- `effigy test compile`: pass
- construction prefix: `1/1` pass
- structural prefix: `18` run, `17` pass, `1` fail
- `S01`-`S16`: pass
- `S17`: fail, `duration-dependent component arrays are forbidden`
- `S18`: pass
- synthetic owners: not run
- mono listening: not opened
- speaker and independent stereo listening: not opened

No structural numeric-row artifact was emitted before the terminal assertion.
No synthetic row or rendered synthetic output exists, so there is no synthetic
output SHA-256 inventory.

## Decision

The candidate materialized full-duration tonal, transient, residual, and
spectral component arrays. Their capacities derive from source duration. That
violates the brief's monotonic bounded-ring design, `96 MiB`
duration-independent working-state cap, and eviction ownership.

This is an architecture-conformance miss, not a threshold or parameter miss.
The `17` passing structural owners do not establish creative quality.
Synthetic and listening admission remained closed, so no PaulX-like quality or
stereo claim follows from this batch.

The checkpoint was not repaired, tuned, rerun, or reinterpreted. The worktree,
branch, checkpoint reference, private module, tests, and worktree-local build
state were deleted. The shared nextest target was cleaned for
`signal-dsp-stretch`; `2359` files / `316.5 MiB` were removed. `main` retained
documentation only.

## Next Task

Run Batch 31.44 as docs-only bounded-state architecture reassessment. Prove or
reject a complete monotonic schedule for source-component production,
descriptor and covariance lookahead, event lifetime, synthesis consumption,
and eviction under `96 MiB`, without changing the sole map or audible owner
semantics. Freeze one fresh complete authority or close
`LinkedStnNoiseMorph`. Do not implement another candidate.
