# g10.032 Centred Cyclic Comparator Recovery

Date: 2026-07-24
Status: Batch 32.16 complete; checkpoint rejected

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

## Result

All `30` comparator rows matched their frozen source, project, output
container, and PCM identities. REAPER reproduced exact PCM; its wall-clock BWF
field was restored to the original value selected by each frozen container
hash.

The unchanged checkpoint passed the first `12` Y01 rows. It failed
`Y01-012-impulse-r2-c048000` with one unexpected dropout. Receipt SHA-256:
`64eec35d2fef5d7ef3c1d219020d901cff864437469c977680558972c34e7529`.
No summary or later row exists.

This is a valid checkpoint rejection. The Cyclic product target remains open
for architecture reassessment.

## Next Task

Execute Batch 32.17 only. Attribute the impulse dropout at complete-system
level. Do not implement or rerun acoustic evidence.
