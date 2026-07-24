# g10.032 Cyclic Acoustic Admission

Batch 32.24 recovered checkpoint `995ea516` exactly into the isolated
`signal-candidate-32-24` worktree. Renderer bytes matched the checkpoint.

Evidence repair made the runner executable and replaced placeholder acoustic
values with measured pitch, event, cadence, gap, tail, balance, and
correlation diagnostics. A comparator event-centre bug and an impossible
cadence-order aggregate were corrected. Renderer formulas did not change.

Results:

- structural round 1: `340/340`
- structural round 2: `340/340`
- synthetic: `183/183`, `201` renders
- exact `16x` rejection: `5/5`, zero output allocation
- long-form mono: `45/45`

The operator pack has `15` concealed neutral A/B rows against ReaReaRea and
`15` Signal short/neutral/long direction rows across the five musical sources
at `2x`, `4x`, and `8x`. Batch 32.25 is active at listening authority.

No candidate DSP entered `main`. Nothing was pushed.
