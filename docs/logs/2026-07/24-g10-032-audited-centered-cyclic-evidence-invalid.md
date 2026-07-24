# g10.032 Audited Centered Cyclic Evidence-Invalid Stop

Date: 2026-07-24
Status: Batch 32.12 stopped; Batch 32.13 ready

## Result

Verified clean checkpoint `74a6d6d9`, tree `d519e2d8`, the three frozen
manifest hashes, an absent `Y01` execution root, and exactly `30` planned
rows. Invoked `Y01` once.

Execution stopped on `Y01-000-low-tone-r2-c048000`:

- the row process reported one passing test
- the shell runner found no receipt at the intended root
- runner exit: `66`
- intended root: environment identity files only
- misplaced root: one complete passing row receipt
- `Y01` summary: absent
- later rows and gates: absent
- retry: not performed

The frozen repo-relative ignored root was forwarded unchanged to nextest.
Nextest ran the test from `crates/signal-dsp-stretch`, so the receipt was
created under a crate-relative duplicate root. The shell runner checked the
repository-relative root and stopped.

Misplaced receipt SHA-256:
`f9c12e26ca6d7e727749ae12e70e86262816715abad66850396ea6fdc4596d91`.

## Decision

The out-of-root pass does not admit the row. This is an evidence-path
ownership failure, not an acoustic quality result. Contract `085` forbids a
retry. It is the second incomplete-evidence checkpoint for centred
compressed-anchor Cyclic, so Rule 11 requires closure.

The isolated worktree, branch, build state, and generated evidence are
deleted after this closeout. The acoustic ref remains only through the
required Batch 32.13 closure reassessment.

## Next Task

Execute Batch 32.13 only. Record final family closure and delete the retained
acoustic ref. Docs only. Do not repair the runner, retry `Y01`, or authorize a
third identity.
