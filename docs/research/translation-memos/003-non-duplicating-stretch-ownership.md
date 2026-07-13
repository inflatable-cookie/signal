# Non-Duplicating Stretch Ownership

Status: promoted
Memo: `g10.029` representation reset
Owner: dsp
Last updated: 2026-07-13
Related roadmap: `g10.029`

## 1) Problem

Signal's simultaneous `512/2048/8192` union assigns the complete source to
three independently windowed coefficient systems. Independent phase transport
and one shared physical-frequency phase field both fail. The shared-field proof
still leaves `162.261364` frames of mean layer-arrival disagreement, `0.134045`
pairwise correlation, and positive replica growth.

The next representation must retain short event support and long tonal support
without synthesizing multiple full-band copies of the source.

## 2) External Evidence

### Complementary source subbands

Perfect-reconstruction filter banks can give every source frequency one
analysis and synthesis path. This is strong ownership for a fixed frequency
partition. It is weak event-local resolution: a broadband event still inherits
each band's fixed time support. Making the bank time-varying moves the hard part
to coupled analysis/synthesis transitions. Primary time-varying filter-bank
work warns that ordinary lossless and analysis/synthesis interchange properties
do not transfer automatically. Filter-bank time scaling also needs
signal-dependent conditions for multi-component signals.

This family does not provide the shortest route to one event-local transform,
one global time map, and exact transition reconstruction.

### Explicit coefficient-plane partitioning

Quilted Gabor frames select local pieces from different Gabor systems in the
time-frequency plane. Proven frame cases are reconstructable, and reduced
multi-window cases admit immediate partition-of-unity reconstruction. Generic
quilts otherwise require an exact dual or iterative approximation; the paper's
cheap reconstruction examples are not one-step perfect reconstruction.

The family can avoid complete full-band copies, but it leaves Signal to invent
and validate quilt overlap, exact local dual support, resolution-boundary phase
transport, and finite-source boundaries together. It is a valid reserve
architecture, not the first implementation target.

### One invertible adaptive-resolution representation

Nonstationary Gabor frames vary windows and lattices while retaining a frame
reconstruction model. Painless systems make the frame operator diagonal, so
the exact dual remains local and explicit. Ottosen and Dörfler apply this class
directly to time stretching, using short transient support, long sinusoidal
support, adaptive phase locking, and transient phase treatment at roughly
three coefficients per sample.

Signal has stronger project-local evidence for this family than for the other
two. Batch 29.6AI already proves declared `512/1024/2048/4096` schedules,
compact support, exact diagonal-dual reconstruction below `1e-15`, and adaptive
condition `1.5934675721`. Batch 29.6BA does not invalidate that representation.
It rejects a fixed-ratio oracle renderer that prohibited local timing and event
phase reset, then placed one `1.5x` impulse `127` frames early. Later Rubber
Band forensics and complete-system work establish that study, exact local time
allocation, event phase treatment, and vertical phase policy must cooperate.

## 3) Recommendation

Select one time-adaptive painless nonstationary Gabor frame.

- one legal resolution is selected at each analysis centre
- adjacent centres form one covering frame; no additive resolution layers exist
- the exact diagonal dual normalizes the complete selected schedule
- one globally exact local time map places every synthesis centre
- event correction follows ordinary actual-hop phase transport
- linked channels share study, resolution, time-map, peak ownership, and reset
  decisions while retaining channel spectra and interchannel phase

Reuse the proven four-window family and legal adjacent-level transition rules.
Reuse the complete-system offline study, selected exact points, and constrained
global schedule. Do not reopen automatic resolution detection: selected event
regions deterministically request the shortest legal support; distance from
protected support permits monotone transitions toward longer windows. This is
a geometry rule, not a corpus-fitted mask.

## 4) Accepted Tradeoffs

- resolution is time-adaptive, not independently frequency-adaptive
- ordinary frame overlap remains; duplicate full-band synthesis layers do not
- exact diagonal-dual normalization may vary with the selected schedule
- offline study and a complete source schedule remain prerequisites
- the first candidate may cost more than current Signal but must remain bounded

## 5) Frozen Contract

Ownership:

- every analysis centre owns exactly one window, FFT, and coefficient vector
- resolution changes use only the proven legal adjacent-level schedule
- no coefficient masks, layer crossfades, complementary renders, or union dual
- the selected windows must cover every cropped output sample with a positive,
  finite diagonal frame operator

Time and phase:

- all coefficients use one positive-integer, globally exact output-hop schedule
- resolution boundaries do not create a second time map or automatic reset
- ordinary phase advances by analyzed instantaneous frequency and actual output
  hop on a physical-frequency topology
- peak-region vertical locking occurs inside the current selected frame
- selected event correction is one downstream operation and uses analyzed phase
  at the projected source point

Boundary and stereo:

- whole-sample even reflection supplies bounded source support
- synthesis includes every frame whose support touches the exact crop
- exact requested length comes from the schedule and crop, never zero fill
- linked stereo shares all discrete decisions and preserves per-channel complex
  coefficients and interchannel phase offsets

No corpus sweep may choose window sizes, transitions, event guards, time-map
weights, reset scope, or phase topology. Existing frozen values transfer only
where this memo names them.

## 6) Required Proof Sequence

1. Re-express the passing declared-schedule reconstruction as single-owner
   invariants. Prove coefficient uniqueness, coverage, condition, identity,
   reflection, real closure, deterministic repeat, and bounded work.
2. Attach the frozen complete-system study and output-hop schedule. Prove exact
   duration, selected-point displacement bounds, and one mapping for all
   resolutions without phase modification.
3. Add ordinary actual-hop phase transport, selected event correction, and
   in-frame vertical locking. Apply impulse placement, replica, tonal, chirp,
   stereo, boundary, and identity gates before corpus audio.
4. Run the existing development corpus once. Concealed passage may open the
   still-sealed holdout; failure returns to representation attribution, not a
   parameter sweep.

## 7) Promotion

Promoted into:

- `docs/architecture/offline-time-stretch-synthesis.md`
- contract `082`, Rule 30K
- roadmap `g10.029`, Batch 29.6BP onward

## 8) Sources

| Source | Confidence | Transfer boundary |
| --- | --- | --- |
| [Phoong and Vaidyanathan, 1996](https://doi.org/10.1109/78.553472) | high | Time-varying filter-bank PR coupling; no Signal filter bank design |
| [Quatieri and Hanna, 1999](https://www.ll.mit.edu/r-d/publications/perfect-reconstruction-time-scaling-filterbanks) | high | Multi-component time-scaling limits; no filter-bank implementation |
| [Dörfler, 2010](https://arxiv.org/abs/0912.2363) | high | Quilted-frame and reconstruction boundaries; no tiling policy |
| [Dörfler and Matusiak, 2012](https://arxiv.org/abs/1112.5262) | high | Nonstationary Gabor frame existence and painless relation |
| [Rudoy, Basu, and Wolfe, 2010](https://arxiv.org/abs/0906.5202) | high | Ordered adaptive window ownership and fast overlap-add reconstruction |
| [Ottosen and Dörfler, 2017](https://arxiv.org/abs/1612.05156) | high | NSG time stretching, adaptive phase locking, and transient treatment |

## Next Task

Execute Batch 29.6BQ study and time-map attachment proof. Keep coefficient and
phase modification, corpus audio, holdout, and tuning closed.
