# g10.031 Bounded Linked STN Capacity Reconciliation

Date: 2026-07-21
Batch: 31.46
Status: complete; capacity-audited v3 candidate ready

## Scope

Resolve the first-residual `53248` formula maximum against the frozen `59392`
row. Change documentation only. Do not recover or implement Batch 31.45.

## Decision

Retain the per-geometry capacity formula:

`N_t+2(h_s*A_s+N_s)`, with `h_s=(R_h-1)/2` for the current request geometry.

Exhaustive integer evaluation over `F=8000..192000` reaches `53248` at:

- `F=192000`
- `N_t=32768`
- `N_s=4096`
- `A_s=1024`
- `R_h=13`
- `h_s=6`

The rejected `59392` row used global `h_s=9` from `F=18000`, `N_t=2048`
with the maximum transform geometry. No supported request has that combination.
Using it would require a different global-half-width allocation formula and
would add `6144` unused samples at maximum geometry.

## Memory Result

The exhaustive capacity row is:

`[17,97,19,57,20,22,53248,147712,98816,39,32772,139520]`

The conservative short/source packed model changes from `9.841 MiB` to
`9.700 MiB`. The `12 MiB` category ceiling remains sufficient. Category
ceilings still total `89 MiB`; `7 MiB` remains unassigned below the unchanged
`96 MiB` terminal actual-allocation gate.

No source lookahead, last consumer, eviction rule, allocation assertion, or
audible owner changes. Every other exhaustive row already matched.

## Fresh Authority

The canonical brief now freezes:

- candidate: `CapacityAuditedBoundedLinkedStnNoiseMorph`
- worktree: `signal-candidate-31-47`
- branch:
  `candidate/g10-031-capacity-audited-bounded-linked-stn-noise-morph`
- private module:
  `creative_capacity_audited_bounded_linked_stn_noise_morph`
- corrected first-residual maximum `53248`
- unchanged `28` structural and synthetic owners

This identity is not a repair, reconstruction, or retry of deleted Batch 31.45
source. Construction must freeze a fresh checkpoint before objective admission.

## Repository Result

- canonical brief, roadmap, front doors, and historical correction notes
  updated
- no DSP, test, harness, fixture, dependency, API, route, cache, artifact,
  product, Loophole, or Chorus change
- unrelated pre-existing plugin worktree changes preserved and unstaged

## Validation

- `git diff --check`: pass
- `effigy qa:docs`: pass
- `effigy qa:northstar`: pass
- `effigy health`: pass
- `effigy validate`: pass
- `effigy doctor`: expected pre-existing `57` god-file and `5`
  attention-marker findings only

## Next Task

Run Batch 31.47 only in `signal-candidate-31-47` on the fresh branch above.
Implement capacity-audited v3 once, complete compile and construction, freeze
one checkpoint, then run structural and synthetic admission in order. Stop
before listening on any miss. Do not recover Batch 31.45, merge, or push.
