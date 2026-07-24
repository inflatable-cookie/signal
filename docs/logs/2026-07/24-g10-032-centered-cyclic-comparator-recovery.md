# g10.032 Centred Cyclic Comparator Recovery

Date: 2026-07-24
Status: Batch 32.15 stopped before DSP; Batch 32.16 ready

## Finding

The absolute-root correction worked. The unchanged runner and test agreed on
the canonical root. The first row then stopped because Batch 32.13 cleanup had
removed the ignored comparator assets:

- missing `comparator/sources/low-tone.wav`
- terminal two-line receipt, SHA-256
  `7b16ffc67d6356b8a47d4fb57828017c0b7fe31a6a98231375e6bc0950129cf5`
- every acoustic assertion `not_run`
- no renderer invocation, render, or summary

This does not evaluate the candidate.

## Recovery

Contract `085` authorizes one exact preparation recovery. Preserve the failed
execution directory, regenerate the canonical synthetic source set and the
`30` frozen `C-Y-*` ReaReaRea comparator rows, verify every frozen hash, and
replay `Y01` once from unchanged checkpoint `74a6d6d9`.

## Next Task

Execute Batch 32.16 only. Stop before `Y02`.
