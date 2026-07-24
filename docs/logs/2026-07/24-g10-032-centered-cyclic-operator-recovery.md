# g10.032 Centred Cyclic Operator Recovery

Date: 2026-07-24
Status: Batch 32.15 stopped before DSP; Batch 32.16 ready

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

## Environment Stop

Batch 32.15 used the exact absolute root. It then found the generated
comparator environment absent after Batch 32.13 cleanup. The first row stopped
on missing `comparator/sources/low-tone.wav` before source decode or renderer
execution. Its receipt is terminal only as an invalid preparation record:
every assertion is `not_run`, and no render or summary exists.

Batch 32.16 preserves that receipt, regenerates and hash-verifies the exact
synthetic sources and `30` frozen ReaReaRea comparator outputs, then invokes
the unchanged Y01 runner once. No candidate or evidence-owner byte changes.

## Next Task

Execute Batch 32.16 only. Restore the exact synthetic comparator environment,
verify it against the frozen manifest, and replay all `30` `Y01` rows once.
Stop before `Y02`.
