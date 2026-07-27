# 2026-07-27 g10.039 Batch 39.1 State Boundary Audit

Status: complete

Documentation only. No crate source changed. One temporary in-crate probe was
added to `signal-render-plane`, run, and removed.

## State That Resets

Every segment and every chunk constructs a fresh `DraftPhaseVocoder`, so all of
this starts from zero at each join:

| state | role | cost of resetting |
| --- | --- | --- |
| `synthesis_phase` | accumulated output phase per bin | arbitrary phase relationship across the join; the dominant defect |
| `previous_phase` | previous frame's analysis phase per bin | first frame after a join cannot propagate |
| `previous_magnitudes` | spectral-flux baseline | detector cannot fire on the first frame after a join |
| `previous_energy` | energy-rise baseline | same |
| `frame_index == 0` branch | forces phase initialisation | guarantees the reset rather than merely allowing it |
| overlap-add and normalization buffers | windowed accumulation | each render's windup and tail are cropped, so joins share no overlap |

The two detector baselines are the part the earlier work had not named. A
transient landing on the first frame after a join is undetectable, because the
flux and energy comparisons have nothing to compare against. That is the
leading hypothesis for `A18`, the low-end pops heard in **both** sides of the
`g10.036` listening round — both sides were segmented renders, so both carried
the defect.

## Boundary Cost At The Shipped Default

Earlier measurements compared segmentation against itself, or used a small
synthetic source. This one uses the production chunk policy on material long
enough to trigger it: `60` seconds of stereo chord plus a click every `250 ms`,
ratio `1.25`, rendered through the artifact path.

| measurement | value |
| --- | --- |
| chunks | `2` against `1` |
| output frames | `3600000` both |
| correlation | `0.389976` |
| peak sample difference | `1.0752` |
| step at the seam sample | `-240 dBFS` production, `-45.14 dBFS` control |

Two readings matter here.

The seam is flat. The boundary smoother drives the step to `-240 dBFS`, below
the control's own signal step of `-45 dB`. Sample continuity is achieved.

The renders still diverge, correlation `0.39`. Continuity of value is not
continuity of phase. The smoother makes the join inaudible as a click while
leaving the two halves phase-unrelated, which is exactly the failure the
`g10.036` listening rounds heard and the reason segment-length tuning could
never fix it.

Every export longer than the `30`-second chunk policy carries this today.

## Contract Amendment

Contract `046` gains the resumable offline render boundary: the carried-state
list, exact chunk-size independence as the acceptance law, a geometry-derived
memory bound, ratio-curve ownership, and the requirement that both seam
mechanisms be *removed* rather than retained.

Chunk-size independence is frozen as exact rather than tolerance-bounded. A
tolerance would readmit the defect the lane exists to remove.

## Ratio Curve Ownership

Decided: the renderer consumes the ratio curve directly.

Caller-side segmentation is what creates the joins. A state-carrying renderer
needs the active ratio per analysis frame, not a pre-cut list of independent
spans, so keeping segmentation caller-side would preserve the boundaries the
lane exists to delete.

`plan_offline_stretch_chunks` remains the bounded-memory authority and stops
being a segmentation authority. That also closes the scope boundary `g10.036`
recorded, where the chunk plan could still cut sub-window chunks from a dense
curve: once the renderer owns the curve, chunk edges no longer coincide with
ratio changes.

## Consequences For Frozen Behavior

This is not a byte-exact lane. Carrying state changes rendered output for every
dynamic-ratio render and every source longer than one chunk. The
`0.5x..3.0x` byte-exact control from `g10.036` continues to hold only for
single-chunk static-ratio renders, and Batch 39.2 must state which controls
survive before implementation opens.

The behavior version and cache schema advance again when this lands, under the
Contract `046` rule that the behavior version moves in the same change as the
output.

## Validation Run

- boundary-cost probe against a whole-buffer control, in-crate, removed after
  use
- `cargo build -p signal-render-plane` after probe removal
- `effigy qa:docs`

## Next Task

Execute `g10.039` Batch 39.2: freeze the resumable render API shape, the state
inventory with a geometry-derived capacity for each item and a total memory
ceiling, the exact equivalence law, and the evidence order. Documentation only,
and it must state which `g10.036` byte-exact controls survive.
