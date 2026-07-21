# g10.031 Construction-Bound V6 Rejection

Date: 2026-07-21
Status: Batch 31.53 complete; candidate rejected at structural admission

## Scope

Implemented `ConstructionBoundZeroPreservingLinkedStnNoiseMorph` once in the
required disposable worktree. Production DSP, APIs, dependencies, routing,
cache, Loophole, and Chorus stayed unchanged.

## Checkpoint

- base: `fdad84326d1d2b576f6a73e96499b77be76dcd4e`
- checkpoint: `366ac24b5cec936209b3e1cbcadafce45eb06bbc`
- tree: `68da7e43784acf8ae1a9d23e77d244153504fd76`
- shape: one private declaration plus eight private files, `3247` lines
- geometry table SHA-256:
  `22d14913f01143007a114fad7a97d44a7e2b07cf5b254b92bc59c7f805e73697`
- geometry FNV-1a-64: `7ffb5aa02900893e`
- `Cargo.lock` SHA-256:
  `e3848a40d2ea1ff88a0e036df40d1fefa56c7aca950a95262c1d8c5668fd394d`
- toolchain: Rust `1.96.0`, `aarch64-apple-darwin`, LLVM `22.1.2`
- platform: Darwin `25.5.0` arm64

| File | SHA-256 |
| --- | --- |
| `lib.rs` | `9b44ae132047d173052d2046675806f28f7b2535f5835e3ce1543576952fa758` |
| `decomposition.rs` | `0e0c5cc59be4ead0476a394bcfa2516db5c22cdf62a8939a7f342516e578e168` |
| `mod.rs` | `1c936ad13e708b07439f54ec476b5bc7e0ff8ba5a41a0eb9f8bd298ff55115f3` |
| `noise.rs` | `037b2731f185d6f6409800f1c8087ae69657a0ef836d0fa6329caf855f2f084b` |
| `plan.rs` | `2aabe0330f3386641d2a3fbbfa99d07dd25973f0c5fc0bc1f9d7f1743e676973` |
| `synthesis.rs` | `f8ad520dc08103d950b3d7b3d2cecaf7c78760e6984de5ad0bf32503a323f9b1` |
| `tests.rs` | `60c9282af2406c7c6c860df87f3b2c3c76e2bb5a756e47673fec0ad10c44b2d4` |
| `tonal.rs` | `c1b91e550d3779b6d447e98f023678ba8fc919249823ad7690ec893ce60c4bc1` |
| `transient.rs` | `b4ebe8813a5d64776c10d052bd938527dc0ca553b1b2d4208c7cb29fc4fe77de` |

The first compile found one Rust move from a shared test vector. The permitted
pre-checkpoint ownership-only repair changed no DSP formula, literal, metric,
threshold, helper result, or assertion. Compile then passed. Construction ran
once and passed `1/1` before the checkpoint was frozen.

## Admission

Structural admission ran once and completed `16/18`:

- `S06` returned peak bins `[1,3,4]`; the frozen equal-plateau tie law requires
  `[1,3]`
- `S18` found the forbidden `pub fn` token in private candidate source
- every other structural row passed

Synthetic and listening gates did not open. The dominant cause is incomplete
construction ownership of structural semantics: geometry was fully bound, but
peak-plateau ownership and private-surface containment were not proved before
checkpoint.

The checkpoint was not repaired or rerun. The worktree, branch, checkpoint
reference, private source, tests, and `3.4 GiB` of build state were deleted.
No candidate DSP entered `main`.

## Next Task

Run Batch 31.54 as docs-only executable-authority reassessment. Either bind
every structural owner into one fresh construction authority or close
`LinkedStnNoiseMorph`. Do not recover checkpoint `366ac24b` or implement DSP.
