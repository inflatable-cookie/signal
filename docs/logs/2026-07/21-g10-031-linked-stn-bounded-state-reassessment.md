# g10.031 Linked STN Bounded-State Reassessment

Date: 2026-07-21
Batch: 31.44
Status: complete; bounded v2 candidate ready

## Scope

Prove or reject bounded execution for the complete `LinkedStnNoiseMorph` owner
graph. Change documentation only. Do not recover the rejected candidate or
start another implementation.

## Baseline

- branch: `main`
- starting commit: `d35d9d10 Reject unbounded linked STN candidate`
- worktree: clean
- rejected Batch 31.43 worktree, branch, checkpoint reference, source, tests,
  and build state: absent by required cleanup
- Northstar posture: baseline routing; no active strict lane

## Decision

The owner graph is bounded. Symmetric medians, long and short WOLA, event
confirmation and capped segmentation, residual covariance, mapped envelope,
tonal tracking, and output normalization all have finite geometry-derived
lookahead and monotonic last consumers.

Residual orientation is the sole non-causal dependency. The stereo law uses
the first exactly non-zero augmented-residual mid and side samples. Bounded v2
runs one deterministic decomposition/event prepass to recover only those two
signs, resets every state owner, then performs the real render. No component,
spectrum, event, descriptor, envelope, or output data crosses the pass
boundary.

The two-pass schedule preserves the sole map and every audible formula. It
changes fixed computational shape, not output ownership.

## Bounded Proof

Exhaustive integer evaluation for every supported sample rate produced:

| Quantity | Maximum | First maximum geometry |
| --- | ---: | --- |
| `Q_h` | `17` | `F=17067`, `N_t=2048` |
| `R_h` | `19` | `F=18000`, `N_t=2048` |
| `Q_v` | `97` | `F=8000`, `N_t=2048` |
| `R_v` | `57` | `F=8000`, `N_t=2048` |

At maximum transform geometry:

| Ring | Maximum |
| --- | ---: |
| native long frames | `20` |
| native short frames | `22` |
| first-residual samples | `59392` |
| transient/residual samples per lane | `147712` |
| claimed-event samples | `98816` |
| live event descriptors | `39` |
| envelope/deque samples | `32772` |
| output finalization samples | `139520` |

Conservative packed-`f64` model rows are `17.502 MiB` long,
`9.841 MiB` short/source WOLA, `4.001 MiB` residual,
`1.508 MiB` event samples, `1.001 MiB` envelope, and `8.516 MiB` output
finalization. Category ceilings total `89 MiB`. The remaining `7 MiB` is
unassigned below the unchanged `96 MiB` terminal actual-allocation gate.

Only immutable input and the returned interleaved `Vec<f32>` may scale with
duration. Full-duration components, spectra, descriptors, event history,
envelopes, denominators, and `f64` output are forbidden.

## Fresh Authority

The canonical brief now freezes:

- candidate: `BoundedLinkedStnNoiseMorph`
- worktree: `signal-candidate-31-45`
- branch: `candidate/g10-031-bounded-linked-stn-noise-morph`
- private module: `creative_bounded_linked_stn_noise_morph`
- unchanged `28` structural/synthetic owners
- compile-linked `MEMORY_SPEC` for frontiers, capacities, budgets, duration
  vectors, two-pass allocation, and source exclusions

This is fresh authority, not a repair or reconstruction of checkpoint
`1c383679`. Quality, `16x`, component leakage, low-frequency residual noise,
linked image, entry/tail character, and computational acceptability remain
terminal candidate risks.

## Repository Result

- architecture, contract, roadmap, front doors, and log updated
- no DSP, tests, harness, fixture, dependency, API, route, cache, artifact,
  product, Loophole, or Chorus change
- `OfflineHighQuality`, RealtimePreview, `g10.028`, and Contract `084`
  unchanged

## Validation

- `git diff --check`: pass
- `effigy qa:docs`: pass
- `effigy qa:northstar`: pass
- `effigy health`: pass
- `effigy validate`: pass
- `effigy doctor`: expected pre-existing god-file and attention-marker
  findings only

## Next Task

Run Batch 31.45 only in `signal-candidate-31-45` on
`candidate/g10-031-bounded-linked-stn-noise-morph`. Implement bounded v2 once,
complete compile and construction, freeze one checkpoint, then run structural
and synthetic admission in order. Stop before listening on any miss. Do not
change production code, merge, touch Loophole or Chorus, or push.
