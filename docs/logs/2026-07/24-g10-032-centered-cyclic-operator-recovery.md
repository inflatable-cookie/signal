# g10.032 Centred Cyclic Operator Recovery

Date: 2026-07-24
Status: Batch 32.14 complete; Batch 32.15 ready

## Correction

The operator rejects closing the renderer over Batch 32.12's invocation-path
mistake.

- checkpoint `74a6d6d9`, tree `d519e2d8`, still exists exactly
- the first `Y01` row passed every assertion
- the shell and nextest process resolved one relative root differently
- no DSP, source, metric, threshold, owner, or valid acoustic receipt failed
- no candidate or evidence byte will change

Contract `085` now authorizes one replay of the same checkpoint with the exact
absolute evidence root. This is not a third candidate and does not permit a
parameter or harness repair.

## Next Task

Execute Batch 32.15 only. Restore the exact isolated identity and acoustic ref,
then run all `30` `Y01` rows once with the frozen absolute root. Stop before
`Y02`.
