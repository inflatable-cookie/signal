# g10.029 Active-Peak And Transient-Anchor Ownership

Date: 2026-07-13

## Scope

Batch 29.6BU replaces dormant-bin phase continuation and resolution-owned event
placement. The painless single-frame transform, exact global schedule,
adaptive synthesis coefficients, diagonal dual, and final duration stay fixed.
The full Rule 30N quality matrix, corpus, holdout, listening, tuning, stereo,
dynamic ratio, cache, and routing remain closed.

## Result

The bounded mechanism proof passes all `32` control/ratio rows.

- all eight hard failure classes are zero
- maximum rendered tone error: `8.211e-7` radians/sample
- maximum matched-owner interior tone error: `5.919e-7` radians/sample
- owner births/matches/retirements: `4,976/46,588/4,960`
- peak-region assignments: `5,204,460`
- expected anchors detected and exactly attached: `24/24`
- maximum identity error: `6.674e-16`
- evidence hash: `a2d3fb95545cb47f`
- predecessor attribution hash: `ddca308a7f60f39e`
- predecessor quality hash: `6781d49348dfa931`

Active spectral peaks own physical frequency and synthesis phase through
ordered one-to-one matching. New owners initialize from current analysis
phase. A fixed analytic tracking spectrum prevents adaptive-window sidelobes
from masquerading as physical peak trajectories without changing synthesis
coefficients or the diagonal dual.

Transient anchors come from independent linked derivative-energy evidence on
the existing `128`-frame grid, refined to sample positions. Every accepted
known-answer anchor becomes an exact source centre at its interpolated global
schedule output.

The dense-event rendered-peak diagnostic still reaches `262` frames. Rule 30P
does not treat that as an ownership failure because detection, attachment, and
one-to-one order are exact. Rule 30Q retains the diagnostic under the complete
unchanged Rule 30N quality limits.

## Decision

Active-peak and transient-anchor ownership are proven. Do not tune them from
this mechanism-only evidence. Open only the full successor synthetic quality
gate.

## Next Task

Execute Batch 29.6BV under Rule 30Q. Run the complete unchanged Rule 30N matrix
through the successor renderer. Return to the earliest owning mechanism on any
hard failure; keep corpus, holdout, listening, stereo, dynamic ratio, and
routing closed.
