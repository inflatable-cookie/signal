# 038 - Stretch Crate Surface And Evidence Consolidation

Status: complete; Batch 38.7 deferred to `g10.039`
Owner: dsp
Created: 2026-07-27
Updated: 2026-07-27
Depends on: `g10.036`, `g10.037`
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
`docs/contracts/084-stretch-candidate-isolation-and-promotion-contract.md`
Vision tags: `DSP`, `STRETCH`, `MAINTAINABILITY`

## Problem

`signal-dsp-stretch` exports `160` public items and `36` of them have a
consumer outside the crate. The audit first reported `249`, which counted every
`pub` declaration including module-internal ones; Batch 38.1 corrected the
count and found the gap wider in proportion than stated. Seven structural
findings sit behind it.

`A12` — the promotion policy is written three times.
`accepts_product_facing_path` encodes the rule set as a boolean,
`product_facing_path_blocker` encodes the same rules as reasons in a different
order, and `product_quality_rejection_note` encodes them a third time with
different wording. Three encodings of one gate can disagree.

`A13` — evidence scaffolding outweighs DSP roughly three to one, and duplicates
itself. `tonal_texture/spectral.rs` and `formant_boundary/spectral.rs` are
near-identical copies of `window_fits`, `hann_window`, and planner setup. Five
modules each construct their own `FftPlanner`.

`A14` — `transient_smear` exposes four public entry points wrapping one
eight-argument private function.

`A15` — `creative_cyclic::render` and `Plan::identity` are `#[cfg(test)]`.
Production calls only `render_continuous`, so tests exercise a path that ships
to nobody.

`A16` — `lib.rs` is `4855` lines, about `2300` of them tests. It holds tier
metadata, the RealtimePreview tier, three stretcher types, dynamic-ratio
segmentation, pitch composition, and the selector gates. The rest of the crate
is already well split; `lib.rs` never was.

`A6` — the offline engine calls `Fft::process` in `analyze_frame` and
`synthesize_frame`, which allocates scratch inside every call: two heap
allocations per STFT frame. The RealtimePreview kernel already uses
`process_with_scratch`.

`A7` — the expansion selector renders the input up to three times to make one
switch decision: the default output, a full draft baseline, then the
short-window render.

`A9` and `A10` are small: `run_phase_vocoder` computes an output offset from
`((prefix_frames - half_window) * ratio + half_window)` where both terms are
`window_size / 2`, so the ratio never participates and the result is always
`window_size / 2`; and two different `wrap_phase` implementations exist, with
the streaming path using the slower one.

## Generation Runway

This lane advances the `g10` runway from *correct and correctly identified*
to *small enough to keep correct*. It is the last lane before the offline
render architecture changes in `g10.039`, so it deliberately lands first and
gives that work a decomposed base.

The visible runway is:

1. surface inventory and retention decision
2. single-owner promotion gate and dead-surface removal
3. metric module consolidation
4. `lib.rs` decomposition
5. hot-loop and selector efficiency, byte-exact
6. closeout

Batch 38.1 is complete: it decided retention for every public item and deferred
the whole RealtimePreview surface to `g10.040`. The next planning checkpoint is
Batch 38.4, where `lib.rs` decomposition must land without disturbing the
`g10.039` base it is meant to prepare.

## Non-Goals

- no change to any rendered output; every batch here is byte-exact
- no RealtimePreview surface removal — `A11` belongs to `g10.040` and this lane
  leaves the RealtimePreview module intact apart from the one tautological test
  named in Batch 38.2
- no renderer, detector, selector-threshold, or algorithm change
- no cache, artifact, routing, runtime, Loophole, or Chorus work
- no new evidence metric

## Goals

- [x] one owner per promotion rule
- [x] one shared STFT surface for the evidence metrics
- [x] one entry point per measurement with an explicit policy argument
- [x] no production path that only exists under `cfg(test)`
- [ ] `lib.rs` decomposed to match the rest of the crate — partial; the
  RealtimePreview tier moved, the rest is Batch 38.7
- [x] no per-frame heap allocation in the offline hot loop
- [x] public surface reduced to what is used, retained deliberately, or
  scheduled by a named roadmap — satisfied by classification, not by reduction

## Execution Plan

### Batch 38.1 - Surface Inventory And Retention Decision

Status: complete

Documentation only. No crate source changed.

- [x] list all public items with consumer, test-only, or unused status
- [x] mark every item retained, removed, or deferred to a named roadmap
- [x] confirm the RealtimePreview surface is deferred to `g10.040` in full
- [x] record the dead enum variants found by the audit —
  `IntegrationMode::CallbackSafeStreaming`,
  `CallbackTimelineMode::SourceProjected`,
  `UnsupportedMode::AudioThreadProcessing`,
  `UnsupportedMode::SourceAdvanceContract`,
  `UnsupportedMode::ChannelLayout`,
  `CallbackProcessError::CallbackProcessingUnsupported` — as `g10.040` inputs,
  not as removals here
- [x] freeze byte-exactness as the acceptance proof for every later batch
- [x] change documentation only

Result:

- count correction: the audit reported `249` public items. That grepped every
  `pub` declaration including module-internal ones. The exported surface is
  `160`, and `36` of them have a consumer outside the crate — a wider gap in
  proportion than the finding stated

| family | external | own tests only | no consumer | total |
| --- | --- | --- | --- | --- |
| evidence | `15` | `0` | `72` | `87` |
| core engine | `6` | `2` | `12` | `20` |
| realtime preview | `0` | `2` | `15` | `17` |
| creative | `0` | `0` | `14` | `14` |
| cache identity | `7` | `0` | `4` | `11` |
| artifact plan | `4` | `0` | `2` | `6` |
| promotion | `4` | `0` | `1` | `5` |
| total | `36` | `4` | `120` | `160` |

- evidence is the reduction target: `72` of `87` unused, one public entry point
  per measurement variant. Batches 38.2 and 38.3 collapse it
- realtime preview deferred whole to `g10.040`, including the six
  never-constructed variants, because that lane decides completion or closure
  and closure is what removes them
- creative retained: it is the Contract `085` public product surface and its
  intended consumer is outside this repository, so no in-repo caller is
  expected
- cache identity, artifact plan, promotion, and the core geometry and tier
  vocabulary retained as contract surfaces
- process-global test state sweep run rather than assumed. The pattern is only
  unsafe with more than one test per binary; the two defects already fixed were
  the only such cases. Three integration-test allocators are safe today but
  fragile, and Batch 38.2 converts them. The sweep does **not** explain `A19`,
  which has no process-global test counter behind it and stays untriaged

### Batch 38.2 - Promotion Gate And Dead Surface

Status: complete

- [x] reduce the promotion policy to one owner so the boolean form derives from
  the reason form
- [x] prove the three previous encodings agreed, or record where they did not
- [x] remove `fft_plans_ready`, which returns `Arc::strong_count(..) >= 1` and
  can never be false, and the assertion that depends on it
- [x] remove the `cfg(test)`-only `creative_cyclic` render and identity paths,
  or make them production paths with a stated caller
- [x] convert the three integration-test allocator counters to thread-local
  state so a second test in those binaries cannot break the measurement
  silently
- [x] apply the Batch 38.1 removals

Result:

- the three encodings agreed on all `1024` receipt shapes exercised before
  consolidation, so the change is behavior-preserving and the promotion tests
  pass unchanged. The duplication was still worth removing: three encodings
  that agree today are three places to edit tomorrow
- `ProductFacingBlocker` is the single gate owner. The boolean form derives
  from the reason form, and both reason wordings are retained as renderings of
  one enum, because the rule must not be duplicated while phrasing legitimately
  differs by surface
- `fft_plans_ready` and its assertion are gone
- `creative_cyclic::render` and `Plan::identity` are gone. Their owner asserted
  that an identity request returns the input verbatim, which production never
  serves — `render_continuous` admits `2N..=8N` and rejects `target == N`. The
  owner now asserts the rejection, and all `15` cyclic call sites use the
  production entry point
- the three integration-test allocator counters are thread-local. A first
  mechanical pass over-reached into genuine cross-thread `Arc<Atomic*>` state in
  two files; both were reverted and redone against the named statics only
- new finding `A20`: `callback_health_counters_advance_and_infer_xruns` is
  wall-clock dependent. It asserts zero xruns for blocks "far faster than the
  deadline", which does not hold under saturated parallel load. Mechanism
  identified, out of lane, recorded for triage

### Batch 38.3 - Metric Consolidation

Status: complete

- [x] extract one shared windowing and STFT surface for the evidence metrics
- [x] collapse the four `transient_smear` entry points to one plus a policy
  argument
- [x] prove every retained measurement returns identical values before and
  after

Result:

- `spectral_support` owns `window_fits`, `hann_window`, `windowed_magnitudes`,
  and `plan_forward_analysis`. Five planner construction sites became one
  helper, and each caller keeps its own bin selection with the reason recorded
  at the call site
- one `measure_transient_smear` plus `StretchTransientSmearPolicies` replaces
  four entry points. The private eight-argument function is unchanged, so no
  measurement moved
- identity proved by capturing the corpus report before and after: `67` lines,
  `27` comparison rows, byte-for-byte identical. That compares the values the
  promotion gates actually read, which is stronger than per-function assertions

### Batch 38.4 - `lib.rs` Decomposition

Status: complete, with the remaining split deferred to Batch 38.7

- [x] move the RealtimePreview tier into its own module
- [x] move the `lib.rs` test bodies to the module they exercise
- [x] remove the dead ratio term in `run_phase_vocoder`, with a comment
  recording what the offset actually is
- [ ] split tier metadata, the stretcher types, dynamic-ratio segmentation,
  pitch composition, and the selector gates into modules — deferred to Batch
  38.7
- [ ] remove the duplicate `wrap_phase` — not byte-exact, see below
- [x] prove byte-exact output across the full regression surface

Result:

- `realtime_preview` is its own module with its `21` tests. `lib.rs` falls from
  `5181` to `3321` lines
- the dead ratio term is gone: both operands of
  `((prefix_frames - half_window) * ratio + half_window)` are `window_size / 2`,
  so the ratio never participated and the result was always `window_size / 2`.
  Window sizes are powers of two, so the simplification is exactly equal
- the duplicate `wrap_phase` is **retained**. Over a `-50..50` sweep at `1e-4`
  steps, `945158` of `1005319` values differ in bits between the two forms,
  worst delta `2.6e-6`, and at exactly `-PI` they disagree in sign. Unifying
  them moves rendered output and the phase-curvature metric, which this lane's
  byte-exact proof forbids. Both sites now record the measurement, so the
  divergence reads as quantified rather than accidental. `A10` is refined, not
  closed
- the remaining split is deferred because it touches the dynamic-ratio and
  chunked-render code `g10.039` is about to rewrite. Splitting now creates churn
  against that lane for no correctness gain
- corpus report byte-identical across Batches 38.3 and 38.4 together

### Batch 38.5 - Hot Loop And Selector Efficiency

Status: complete

- [x] replace `Fft::process` with `process_with_scratch` in the offline engine
- [x] remove the redundant work in the expansion selector decision
- [x] prove byte-exact output, and record before/after render cost

Result:

- allocations during one `4`-second render at ratio `2.0`, `379` STFT frames,
  measured with a counting allocator: `789` before, `31` after. The difference
  is exactly `2 x 379`, confirming the audit's per-frame claim. Render time
  moved only `24.5 ms` to `22.5 ms`, so the honest description is removal of
  per-frame heap traffic, not a throughput win
- `A7` is partly closed. The three renders are inherent to how the gate is
  defined — measure the current output, then compare its smear against a draft
  baseline — and removing one changes what the gate decides. The duplicated
  *measurement* was removable: both comparisons detected source transients from
  the same input with the same policy and geometry. Detection now runs once,
  taking the selector path from `85.0` to `74.3 ms` at ratio `1.5` and `116.0`
  to `103.3 ms` at ratio `2.0`
- a Batch 38.4 mis-gate surfaced here: the new `realtime_preview` re-export had
  taken over the `#[cfg(any(test, feature = "evidence"))]` attribute belonging
  to the promotion block. Every Batch 38.4 validation command used
  `--all-features` or a multi-crate invocation, so the no-feature build was
  never exercised. The attribute is restored and the no-feature build is now
  part of this lane's validation set
- corpus report byte-identical across Batches 38.3, 38.4, and 38.5

### Batch 38.7 - Remaining `lib.rs` Split

Status: blocked on `g10.039`

- [ ] split tier metadata, the stretcher types, dynamic-ratio segmentation,
  pitch composition, and the selector gates out of `lib.rs`
- [ ] revisit the duplicate `wrap_phase` in a batch that can carry a
  re-baseline with evidence
- [ ] leave `lib.rs` holding crate documentation, module wiring, and public
  re-exports

### Batch 38.6 - Closeout

Status: complete

- [x] run `effigy validate` and the full crate suite
- [x] publish the public surface
- [x] update the `g10` front doors
- [x] name the next ready batch in `g10.039`

| measure | Batch 38.1 | now |
| --- | --- | --- |
| exported items | `160` | `158` |
| external consumer | `36` | `36` |
| no consumer | `120` | `118` |

The exported surface barely moved, and the goal "public surface reduced to what
is used, retained deliberately, or scheduled by a named roadmap" is satisfied by
classification rather than reduction. Everything deletable was deliberately not
deleted: RealtimePreview belongs to `g10.040`, creative is the Contract `085`
product surface, and the evidence family was addressed structurally rather than
by removal.

What the lane did change, all byte-exact:

| area | before | after |
| --- | --- | --- |
| `lib.rs` | `5181` lines | `3343` |
| promotion gate encodings | `3` | `1` |
| duplicated spectral helpers | `2` copies | `1` shared module |
| planner construction sites | `5` | `1` helper |
| transient-smear entry points | `4` | `1` plus policy |
| allocations per `4 s` render | `789` | `31` |
| expansion selector, ratio `2.0` | `116.0 ms` | `103.3 ms` |
| `cfg(test)`-only production paths | `2` | `0` |
| process-global test counters | `2` unsafe | `0` |

## Acceptance Criteria

- [ ] output is byte-exact against the `g10.036` baselines at every batch
- [ ] one promotion rule owner, with the boolean derived from the reason form
- [ ] no test asserts a condition that cannot fail
- [ ] no `cfg(test)`-only production path remains
- [ ] no module builds its own planner where a shared one exists
- [ ] `lib.rs` holds crate documentation, module wiring, and public re-exports
- [ ] zero per-frame heap allocations in the offline hot loop, proven
- [ ] every remaining public item has a consumer, a retention reason, or a
  named roadmap

## Risks and Mitigations

- Risk: consolidation changes a measurement and silently moves a gate.
  Mitigation: Batch 38.3 proves identical values for every retained metric
  before the old code is removed.
- Risk: deleting surface that `g10.040` needs. Mitigation: Batch 38.1 defers
  the whole RealtimePreview surface, and this lane touches only the one test
  helper that cannot fail.
- Risk: `process_with_scratch` changes results. Mitigation: it is the same
  transform with caller-owned scratch; byte-exactness is the acceptance proof,
  and the batch stops if it does not hold.
- Risk: decomposition churn collides with `g10.039`. Mitigation: this lane
  lands first by design and `g10.039` starts from the decomposed tree.

## Evidence Requirements

- [ ] one log per completed batch under `docs/logs/`
- [ ] the Batch 38.1 public-surface inventory table
- [ ] byte-exactness proof per batch
- [ ] metric-equality proof for Batch 38.3
- [ ] render time and peak heap before/after for Batch 38.5
- [ ] commands actually run

## Next Task

Execute `g10.039` Batch 39.1: enumerate every piece of renderer state that
resets at a chunk or dynamic-ratio boundary and what each reset costs audibly,
measure the chunked artifact path against a whole-buffer control, amend Contract
`046` with the resumable offline render boundary, and decide whether the ratio
curve is consumed by the renderer or stays a caller-side concern. Documentation
only.

Batch 38.7 reopens after `g10.039` settles the render architecture.
