# 036 - Transparent Stretch Correctness Recovery

Status: planned; Batch 36.1 ready
Owner: dsp
Created: 2026-07-27
Depends on: `g10.030`, `g10.035`
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
`docs/contracts/084-stretch-candidate-isolation-and-promotion-contract.md`
Vision tags: `DSP`, `STRETCH`, `CORRECTNESS`

## Problem

The 2026-07-27 audit of `signal-dsp-stretch` measured four defects in the
retained Transparent baseline and one defect in its evidence harness. None are
quality opinions; each was reproduced with a runnable probe against the
public API.

`A1` — output dropouts above `4x`. Synthesis hop is `analysis_hop * ratio`.
At the frozen `2048/512` geometry any ratio above `4.0` drives the synthesis
hop to or past the window, overlap-add coverage disappears, and the `1.0e-3`
normalization gate zeroes samples. Measured on a 440 Hz 48 kHz tone, 512-frame
RMS blocks, interior only:

| ratio | near-zero blocks | min block RMS |
| --- | --- | --- |
| `2.0` | `0/172` | `0.696` |
| `4.0` | `0/359` | `0.612` |
| `6.0` | `183/547` | `0.000` |
| `8.0` | `368/734` | `0.000` |

`sanitize_ratio` accepts every finite positive ratio, so the API admits inputs
the renderer cannot serve and reports no degradation.

`A2` — dense ratio curves silently become varispeed. `stretch_dynamic_ratio_*`
renders each segment independently, and any segment shorter than `window_size`
falls through to `linear_time_scale`, which is time-domain interpolation and
therefore pitch-shifts. A curve of `47` points spaced `1024` source frames apart
at ratio `2.0` produced a dominant frequency of `220.0 Hz` from a `440 Hz`
source, against `440.0 Hz` for the same ratio without a curve. Pitch
preservation, the defining Transparent promise, is lost with no error.

`A3` — mono dynamic-ratio renders never smooth their segment seams.
`stretch_dynamic_ratio_linked_stereo_with_engine` calls
`smooth_dynamic_segment_boundaries_interleaved`; the mono engine path does not.
Same source, same curve, one boundary:

| path | seam click |
| --- | --- |
| mono | `-28.940011 dBFS` |
| linked stereo | `-180.617997 dBFS` |

`A4` — no output-size bound. `TimeStretcher::stretch_mono` returns `Vec` and
cannot refuse. Ratio `1.0e6` over `4096` input frames allocated `4096000000`
samples and returned after roughly one minute.

`A17` — the creative allocation gate is thread-unsafe. The
`direct_renewal_dream` test module installs a process-global
`#[global_allocator]` whose measuring flag and counters are plain statics. Its
mutex serializes only tests that take it, so every other test thread's
allocations are counted. `cargo test -p signal-dsp-stretch` failed
`direct_renewal_dream_structural_allocation_memory` with `53693` counted
allocations against a required `0`; the same test passes alone. Until this is
repaired, no full-suite run is trustworthy evidence for any later batch.

## Generation Runway

This lane reopens Signal-only stretch execution after `g10.035` closed with no
ready batch. It advances the `g10` runway from *explicit product matrix
published* to *the published matrix is actually honored by the renderer*.

The visible runway is:

1. defect authority and contract amendment
2. evidence-integrity repair and failing regression owners
3. overlap and ratio-envelope correction
4. dynamic-ratio segment and seam correction
5. re-baseline, full validation, closeout

`g10.037` (cache identity), `g10.038` (surface consolidation), `g10.039`
(resumable offline render), and `g10.040` (RealtimePreview completion) follow.
The next planning checkpoint is Batch 36.4, where the first audible change
inside the retained `0.5x..4x` product range requires an explicit promotion
decision.

## Goals

- [ ] make the renderer's real behavior match the published Transparent product
  range
- [ ] restore pitch preservation for every admitted dynamic ratio curve
- [ ] give mono and linked stereo the same seam treatment
- [ ] bound output allocation at the API boundary
- [ ] make the full test suite a trustworthy evidence surface again
- [ ] keep output byte-exact inside `0.5x..4x` wherever a correction does not
  require an audible change

## Non-Goals

- no new renderer, successor family, or algorithm research
- no change to Dream, Cyclic, or any creative surface
- no cache identity, artifact, routing, or RealtimePreview work
- no formant, tail-envelope, or transient-detector tuning
- no external production dependency
- no relaxation of Contract `084` evidence order for audible changes

## Correction Boundary

Two classes of correction, judged differently:

- **extension**: behavior changes only outside the retained `0.5x..4x` product
  range or only for inputs the renderer previously destroyed. Byte-exact output
  inside `0.5x..4x` is mandatory and is the acceptance proof.
- **audible correction**: behavior changes inside the product range. `A2` and
  `A3` are both in this class. These require objective evidence plus concealed
  listening before admission, under Contract `084` Rule 5.

`A1` is an extension: adapting the analysis hop only when
`analysis_hop * ratio` would exceed the safe overlap bound leaves every ratio
in `0.5x..4x` untouched at the frozen `2048/512` geometry.

## Execution Plan

### Batch 36.1 - Defect Authority And Contract Amendment

Status: ready

Documentation only.

- [ ] record all five defects with their reproduced measurements as durable
  evidence
- [ ] amend Contract `046` to freeze the supported ratio envelope for the
  Transparent renderer and the relationship between window, analysis hop, and
  synthesis hop that the envelope depends on
- [ ] amend Contract `046` to freeze a minimum dynamic-ratio segment law so no
  admitted curve can route a segment into the sub-window interpolation fallback
- [ ] amend Contract `046` to require seam treatment parity across channel
  counts
- [ ] amend Contract `084` so defect correction of the frozen baseline is
  authorized after successor closure, and state which byte-exact regression
  baselines may be re-frozen and on what evidence
- [ ] decide the `A1` policy: enforce the published `0.5x..4x` range at the API,
  extend renderer support above it with a ratio-adaptive hop, or both
  (recommendation: both — enforce the honest range and stop corrupting inputs
  above it)
- [ ] decide the `A4` bound: maximum output frames, and whether `TimeStretcher`
  gains a fallible entry point or the bound is enforced before dispatch
- [ ] classify each correction as extension or audible correction
- [ ] change documentation only

Stop condition: if the operator rejects any audible correction inside the
product range, Batch 36.4 does not open and the defect is recorded as accepted
behavior.

### Batch 36.2 - Evidence Integrity And Failing Owners

Status: blocked on Batch 36.1

- [ ] repair `A17`: make the allocation measuring flag and counters
  thread-scoped so only the measuring thread's allocations are counted, or move
  the gate to a dedicated single-threaded execution owner
- [ ] prove the repair by running the full crate suite twice with unchanged
  results
- [ ] add the failing regression owners before any renderer change: overlap
  coverage at ratios above `4x`, dense-curve pitch preservation, mono/stereo
  seam parity, and the output-size bound
- [ ] record each new owner's expected pre-fix failure

### Batch 36.3 - Overlap And Ratio Envelope Correction

Status: blocked on Batch 36.2

- [ ] implement the Batch 36.1 `A1` decision
- [ ] implement the Batch 36.1 `A4` bound
- [ ] prove byte-exact output inside `0.5x..4x` against the retained baselines
- [ ] prove overlap coverage has no zeroed interior block at the newly
  supported ratios
- [ ] leave `A2` and `A3` failing

### Batch 36.4 - Dynamic Ratio Segment And Seam Correction

Status: blocked on Batch 36.3

- [ ] coalesce or resample dynamic-ratio segments so no admitted curve produces
  a sub-window segment
- [ ] give the mono dynamic-ratio path the same seam treatment as linked stereo
- [ ] measure dense-curve pitch error and mono/stereo seam click before and
  after
- [ ] run concealed listening on dynamic-ratio material before admission
- [ ] record the re-baselined byte-exact hashes with the evidence that
  justified each change

### Batch 36.5 - Re-Baseline And Closeout

Status: blocked on Batch 36.4

- [ ] run the full corpus comparison and acceptance report
- [ ] run `effigy validate` and the full crate suite
- [ ] update Contract `046`, Contract `084`, and the `g10` front doors to the
  corrected behavior
- [ ] publish the corrected Transparent behavior matrix
- [ ] name the next ready batch in `g10.037`

## Acceptance Criteria

- [ ] no interior output block is zeroed at any ratio the API accepts
- [ ] every admitted dynamic-ratio curve preserves pitch within the frozen
  Transparent tolerance
- [ ] mono and linked-stereo seam click agree within one frozen tolerance
- [ ] output allocation is bounded and over-large requests are refused, not
  attempted
- [ ] the full crate suite passes twice with identical results
- [ ] byte-exact output inside `0.5x..4x` for every extension-class correction
- [ ] every audible correction carries objective rows plus concealed listening
- [ ] contracts, roadmap, and front doors agree with shipped behavior

## Risks and Mitigations

- Risk: correcting the frozen baseline reopens successor research by the back
  door. Mitigation: Batch 36.1 authorizes defect correction only; no new
  family, detector, or window variant is in scope.
- Risk: re-baselining byte-exact hashes hides an unintended change.
  Mitigation: only Batch 36.4 may re-freeze a hash, and only with the objective
  rows and listening evidence that justified it recorded alongside.
- Risk: the seam fix trades a click for a low-frequency artifact, since the
  current smoother is a decaying DC nudge from two edge samples rather than a
  crossfade. Mitigation: Batch 36.4 measures both; the durable replacement is
  `g10.039`, and Batch 36.4 states explicitly that it ships parity, not the
  final mechanism.
- Risk: the allocation-gate repair masks a real regression. Mitigation: prove
  the repaired gate still fails when a deliberate allocation is introduced.

## Evidence Requirements

- [ ] one log per completed batch under `docs/logs/2026-07/`
- [ ] the intake log recording all five defects with reproduced measurements
- [ ] before/after tables for dropout blocks, dense-curve pitch, and seam click
- [ ] byte-exactness proof for extension-class corrections
- [ ] concealed listening record for audible corrections
- [ ] commands actually run, including two full-suite runs

## Next Task

Execute Batch 36.1. It is documentation only: record the defect authority,
amend Contracts `046` and `084`, and decide the ratio-envelope, output-bound,
and correction-class questions. No code changes in that batch.
