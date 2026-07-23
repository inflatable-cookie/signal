# g10.032 Centered Cyclic Evidence-Invalid Stop

Date: 2026-07-23
Status: Batch 32.6 stopped; Batch 32.7 complete

## Result

Ran `Y01` once from immutable checkpoint
`4600d228286797d22e4f4d5ca4efa997835fc4b2`, tree
`fa1fc8031a4aab4302b778474702e658784d8a64`.

- nextest run: `1f64f1a3-f8c3-4201-97a3-c7e8f6dbf9dd`
- result: exit `100`
- surfaced error: `unexpected dropout 1`
- receipt: absent
- later gates: not run

No row, source, ratio, output hash, prior completed row, or terminal summary
survived. No candidate WAV or listening pack was written.

## Audit

The frozen executable authority is incomplete:

- owners build all rows before opening the receipt
- error returns panic at `unwrap()` before persistence
- receipt status cannot encode failure
- input/comparator hashes are always null
- assertion arrays do not name actual assertions
- construction does not execute or prove the failure receipt path
- `Y04` omits modulation-peak, sideband, and autocorrelation values
- `Y05` does not measure the frozen gap support
- `Y06` omits mapped-window balance, width, and complete comparator deltas
- exact `16x` covers one structural request, not five retained sources
- no candidate long-form, level-match, concealment, or listening executor
  exists

The earlier structural receipts are byte-identical but not incrementally
durable. They did not prove the Rule 11 boundary required before checkpoint
creation.

## Decision

This is incomplete executable evidence, not a valid renderer-quality result.
The unreceipted dropout cannot guide DSP, source, scalar, metric, or threshold
changes.

Contract `085` permits one fresh audited identity. It must retain the canonical
centred compressed-anchor renderer, start after all old isolated state is
deleted, and bind every row, failure, hash, assertion, `16x` control, and
listening edge before implementation.

## Cleanup

Delete after this docs closeout:

- isolated worktree and branch
- candidate source, tests, ledger, and runner config
- build state and generated comparator copies
- local acoustic evidence ref

No candidate code or evidence scaffolding entered `main`.

## Next Task

Execute Batch 32.8 only. Freeze the complete docs-only
`AuditedCenteredCompressedAnchorCyclic` authority. Do not implement, render,
recapture comparators, or recover rejected source.
