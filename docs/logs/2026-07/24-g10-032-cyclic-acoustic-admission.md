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
- long-form linked stereo: `15/15`

The operator pack has `15` concealed neutral A/B rows against ReaReaRea and
`15` Signal short/neutral/long direction rows across the five musical sources
at `2x`, `4x`, and `8x`. A separate concealed neutral stereo pack contains
the `15` linked-stereo musical A/B rows. Batch 32.25 is active at listening
authority.

The operator judged the concealed outputs hard to distinguish, consistently
similar, and solid. No significant mono or stereo issue was reported. Musical
character therefore passes operator review.

After all hard stereo controls passed, the operator explicitly waived
independent stereo review for checkpoint
`bab6ce96b0476e025dce5c957d91eab27e375fd6`, scoped to fixed `2x`, `4x`,
and `8x`. The operator's one-ear hearing limitation remains recorded. This is
an operator-owned creative-product pass, not a claim that eligible independent
listening occurred. It does not generalize to another renderer, ratio,
character, automatic route, dynamic path, or transparent stretch.

The concealment key was revealed only after the waiver. Candidate placement
was mixed:

- mono `8x`: percussion A, pads A, full mix B
- stereo `8x`: percussion B, pads B, full mix A

Batch 32.26 then admitted only the private production core in commit
`81edaada`. The accepted plan, schedule, interpolation, and synthesis files
are byte-identical to the checkpoint. Six focused tests preserve identity,
fixed-ratio output, deterministic finiteness, linked-stereo algebra, typed
rejection, pre-allocation `16x` rejection, and bounded geometry.

No candidate runner, comparator, receipt, listening pack, public character,
router, cache, artifact, UI, runtime, Loophole, or Chorus surface entered
`main`. Nothing was pushed.
