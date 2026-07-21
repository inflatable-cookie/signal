# g10.031 Bounded Linked STN Construction Rejection

Date: 2026-07-21
Batch: 31.45
Status: complete; candidate rejected before checkpoint and deleted

## Scope

Implement bounded v2 once in its named disposable worktree. Run compile and
construction, freeze one checkpoint only after `1/1`, then run structural and
synthetic admission in order. Stop and delete on any miss.

## Baseline And Isolation

- main start: `8f384c09f30ac189ce9f2b003eea9af588d310e8`
- worktree: `/Users/tom/Dev/projects/signal-candidate-31-45`
- branch: `candidate/g10-031-bounded-linked-stn-noise-morph`
- checkpoint: none; construction never passed
- candidate tree: none
- public API, production DSP, dependencies, routes, cache, artifacts,
  Loophole, and Chorus: unchanged

The attempt contained only the eight required private module files plus one
private `lib.rs` declaration. No candidate source entered `main`.

## Executable Identity

SHA-256 before deletion:

| Surface | Digest |
| --- | --- |
| `decomposition.rs` | `813f4f49c63355c9c171c03832e5944004fe781ee4474fd28c0724eb2294266e` |
| `mod.rs` | `bbe848c6b5be5e83a35c5bdb681ef6654ce05dea3ef5f4620ebc39a5e54ffe69` |
| `noise.rs` | `c8a337aee9da1634ec06798f114f4e6ff2929f163df7a55f245776d579632d80` |
| `plan.rs` | `8f71bf72332d17b9e77aad4f3de91c16a43200a6cca5ad8ab4bd9a32de601821` |
| `synthesis.rs` | `22700764d0adfd83ac51703977423495627b8ac2aaa486876fdbbc5f3ef41f74` |
| `tests.rs`, including `EVIDENCE_SPEC` and `MEMORY_SPEC` | `cf11ae18e24eb56411770012d21a1b0c66cf8600f6130e4921a76920e4165b28` |
| `tonal.rs` | `71fd7167c62fd4432094ffb49bf166249ec1051f79f28efd45a7fcdcf9a3de0e` |
| `transient.rs` | `2417b8c99da3caef400cd426445d7bc644bf16f73eb8274c92ae88e605c82a4c` |
| `Cargo.lock` | `e3848a40d2ea1ff88a0e036df40d1fefa56c7aca950a95262c1d8c5668fd394d` |

No checkpoint froze these files. The digests identify only the stopped
pre-checkpoint attempt and cannot authorize recovery or comparison.

Toolchain and platform:

- `rustc 1.96.0 (ac68faa20 2026-05-25)`
- rustc commit `ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`
- host `aarch64-apple-darwin`; LLVM `22.1.2`
- Darwin `25.5.0`, arm64

## Gate Receipt

- `effigy test compile`: pass after permitted compiler-only assembly fixes
- construction prefix: `1` run, `0` pass, `1` fail
- failure: first-residual exhaustive maximum was `53248`; frozen assertion was
  `59392`
- checkpoint: not created
- structural owners: not run
- synthetic owners: not run
- mono or stereo listening: not opened
- rendered synthetic outputs: none

The complete construction numeric row was:

`[17,97,19,57,20,22,53248,147712,98816,39,32772,139520]`

The frozen row differed only at first-residual samples:

`[17,97,19,57,20,22,59392,147712,98816,39,32772,139520]`

## Cause

The brief defines first-residual capacity as:

`N_t+2(h_s*A_s+N_s)`, with `h_s=(R_h-1)/2` for the current geometry.

At maximum transform geometry, `F=192000`, `N_t=32768`, `N_s=4096`,
`A_s=1024`, `R_h=13`, and `h_s=6`. The formula gives `53248`. Independent
integer evaluation confirmed this is the supported-rate maximum.

The global `R_h=19`, `h_s=9` maximum occurs at `F=18000`, `N_t=2048`, where
the formula gives `3712`. Applying that global half-width to maximum transform
geometry gives `59392`, but mixes two geometries and does not follow the frozen
per-geometry formula.

This is an executable-authority contradiction. It is not evidence about tonal,
transient, residual, stereo, boundary, or listening quality.

## Batch 31.46 Resolution

The retained per-geometry formula is authoritative. Exhaustive evaluation
reaches `53248`; the `59392` row was a cross-geometry composition with no
matching request or consumer. The conservative short/source model is
`9.700 MiB`. Category ceilings remain `89 MiB` with `7 MiB` unassigned below
the `96 MiB` terminal gate.

Fresh `CapacityAuditedBoundedLinkedStnNoiseMorph` identity supersedes only the
deleted candidate identity and erroneous maximum row. Batch 31.45 remains
terminal and unrecoverable.

## Cleanup

No formula, expected value, helper, assertion, DSP constant, or candidate code
was repaired after the miss. The worktree, branch, private renderer, tests, and
worktree-local build state were deleted. The shared nextest target used by the
candidate contained `31524` files / `2.4 GiB`; it was removed from `/tmp` and
moved to the user's Trash as
`signal-nextest-target-31-45-20260721`, so that cleanup remains recoverable.
`main` retained documentation only.

## Validation

- `git diff --check`: pass
- `effigy qa:docs`: pass
- `effigy qa:northstar`: pass
- `effigy health`: pass
- `effigy validate`: pass
- `effigy doctor`: expected pre-existing god-file and attention-marker
  findings only

## Next Task

Historical next task completed in Batch 31.46. Run Batch 31.47 only under the
fresh worktree, branch, module, and test prefixes frozen in the canonical
brief. Do not recover Batch 31.45 source or evidence.
