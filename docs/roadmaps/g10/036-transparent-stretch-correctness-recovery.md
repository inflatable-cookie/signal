# 036 - Transparent Stretch Correctness Recovery

Status: complete; `A2` admitted with a recorded limitation owned by `g10.039`
Owner: dsp
Created: 2026-07-27
Updated: 2026-07-27
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
  over the unaffected range is mandatory and is the acceptance proof.
- **audible correction**: behavior changes inside the product range. These
  require objective evidence plus concealed listening before admission, under
  Contract `084` Rule 5.

Batch 36.1 measured the boundary rather than assuming it, and corrected the
first draft of this section. `A1` is not purely an extension. The frozen
`2048/512` geometry is clean through ratio `3.0`, ripples `1.396 dB` at ratio
`4.0`, and loses coverage entirely above it, so the overlap law's audible
window is `3.0 < ratio <= 4.0` and everything above `4.0` is extension.

Frozen classification:

| defect | class | affected range |
| --- | --- | --- |
| `A1` overlap coverage | audible correction | `3.0 < ratio <= 4.0` |
| `A1` overlap coverage | extension | `ratio > 4.0` |
| `A2` dynamic-ratio segments | audible correction | curves with sub-window spans |
| `A3` seam parity | audible correction | mono dynamic-ratio renders |
| `A4` output bound | extension | refused renders only |
| `A17` allocation gate | neither; test-harness repair | none |

Byte-exactness over `0.5x..3.0x` is therefore the standing control for the
whole lane.

## Execution Plan

### Batch 36.1 - Defect Authority And Contract Amendment

Status: complete

Documentation only. No crate source changed.

- [x] record all five defects with their reproduced measurements as durable
  evidence
- [x] amend Contract `046` to freeze the supported ratio envelope for the
  Transparent renderer and the relationship between window, analysis hop, and
  synthesis hop that the envelope depends on
- [x] amend Contract `046` to freeze a minimum dynamic-ratio segment law so no
  admitted curve can route a segment into the sub-window interpolation fallback
- [x] amend Contract `046` to require seam treatment parity across channel
  counts
- [x] amend Contract `084` so defect correction of the frozen baseline is
  authorized after successor closure, and state which byte-exact regression
  baselines may be re-frozen and on what evidence
- [x] decide the `A1` policy
- [x] decide the `A4` bound: maximum output frames, and whether `TimeStretcher`
  gains a fallible entry point or the bound is enforced before dispatch
- [x] classify each correction as extension or audible correction
- [x] change documentation only

Result:

- the overlap law is `analysis_hop * ratio <= 0.75 * window_size`, with the
  renderer reducing the analysis hop to `floor(0.75 * window_size / ratio)`
  when a configured geometry would exceed it. Measured against both a tone and
  a three-tone broadband source at the frozen `2048/512` geometry
- the law was chosen over the more conservative `window_size / 2` bound
  because `0.75 * window_size` measures identically to full `75%` overlap
  while leaving ratios through `3.0` byte-identical. The conservative bound
  would have disturbed ratio `3.0` for no measured gain
- `A1` is both: the API keeps accepting ratios above `4.0`, and the renderer
  stops destroying them. Enforcing a hard `4x` API ceiling was rejected because
  the chunked artifact path and `g10.039` both need honest behavior above the
  product range
- Contract `084` gains Rule 9, which authorizes defect correction after
  successor closure, and Rule 10, which governs re-freezing byte-exact hashes
- the standing control for the lane is byte-exactness over `0.5x..3.0x`, not
  `0.5x..4x`

- the `A4` bound is `268435456` output samples, one gibibyte of `f32`, and
  `TimeStretcher` becomes fallible to carry it. The operator chose the breaking
  change over a parallel checked entry point so no unbounded path survives in
  the public API. `signal-render-plane` and `signal-runtime` update in the same
  batch; pre-1.0, so no compatibility shim

Stop condition, unchanged: if the operator rejects any audible correction
inside the product range, Batch 36.4 does not open and the defect is recorded
as accepted behavior.

### Batch 36.2 - Evidence Integrity And Failing Owners

Status: complete

- [x] repair `A17`: make the allocation measuring flag and counters
  thread-scoped so only the measuring thread's allocations are counted, or move
  the gate to a dedicated single-threaded execution owner
- [x] prove the repair by running the full crate suite twice with unchanged
  results
- [x] add the failing regression owners before any renderer change: overlap
  coverage and ripple at ratios above `3.0`, dense-curve pitch preservation,
  mono/stereo seam parity, and the output-size bound
- [x] record each new owner's expected pre-fix failure

Result:

- the gate's measuring flag, live/peak byte counters, and allocation counter
  moved from process-global atomics to const-initialized thread-local `Cell`s,
  so the process-global allocator hook attributes allocations only to the
  measuring thread. Const init registers no destructor, so reading the state
  from inside the allocator cannot re-enter it
- the peak is now measurement-local rather than a global high-water mark minus
  a baseline, which removes the last cross-thread term
- the now-pointless `ALLOCATION_LOCK` mutex is gone; thread-scoped state makes
  serialization unnecessary
- two consecutive full-suite runs: `179` lib tests passed, `0` failed, both
  runs, `182.24s` and `181.42s`. Before the repair the same suite failed
  `direct_renewal_dream_structural_allocation_memory` at `53693` counted
  allocations
- negative control: a deliberate `Vec::with_capacity(8)` per render block made
  the gate report exactly `8` processing allocations and fail. The probe was
  reverted and the gate returned to green. The repaired gate detects real
  allocations without counting noise
- five owners land in `crates/signal-dsp-stretch/tests/transparent_correctness_owners.rs`,
  `#[ignore]`d with the owning batch named in each attribute so `main` stays
  green. Two controls are active immediately

Recorded pre-fix failures, from `cargo test ... -- --ignored`:

| owner | pre-fix failure |
| --- | --- |
| `overlap_coverage_has_no_zeroed_interior_block` | ratio `5.0`: `90` interior blocks lost coverage |
| `overlap_ripple_stays_within_ceiling` | ratio `4.0` tone: `1.396 dB` against the `0.5 dB` ceiling |
| `dense_ratio_curve_preserves_pitch` | `220.0 Hz` rendered against `440.0 Hz` |
| `dynamic_ratio_seam_click_matches_across_channel_counts` | mono `-28.940011 dBFS` against stereo `-180.617997 dBFS` |
| `oversized_output_request_is_refused` | not expressible until `TimeStretcher` is fallible |

New datum: ratio `5.0` already loses `90` blocks, so coverage collapse starts
between `4.0` and `5.0` rather than at `6.0` as the audit first recorded.

Active controls, passing now and required to keep passing:

- `overlap_law_leaves_low_ratios_byte_exact` guards determinism and the
  output-length contract over `0.5x..3.0x`
- `dense_ratio_curve_preserves_output_length` proves a dense curve already
  holds output length, so only pitch is broken and coalescing must not regress
  length

### Batch 36.3 - Overlap And Ratio Envelope Correction

Status: complete

- [x] implement the frozen overlap law: reduce the analysis hop to
  `floor(0.75 * window_size / ratio)` when the configured geometry would exceed
  `0.75 * window_size` synthesis hop
- [x] make `TimeStretcher` fallible and enforce the `268435456`-sample ceiling
- [x] update `signal-render-plane` and `signal-runtime` in the same batch
- [x] prove byte-exact output over `0.5x..3.0x` against the retained baselines
- [x] prove no zeroed interior block and ripple at or below `0.5 dB` at every
  ratio the API accepts
- [x] re-baseline only the hashes the `3.0 < ratio <= 4.0` change invalidates,
  under Contract `084` Rule 10
- [x] record render cost at the newly supported ratios
- [x] leave `A2` and `A3` failing

Result:

- `overlap_safe_analysis_hop` lands in `phase_vocoder.rs` and is applied at the
  single choke point every offline engine call passes through, so mono, linked
  stereo, pitch, and dynamic-ratio paths inherit it without duplication
- `TimeStretcher::stretch_mono` and the eleven whole-buffer entry points beside
  it now return `Result<Vec<Sample>, StretchRenderError>`. `73` call sites
  across six files updated. `signal-render-plane` and `signal-runtime` build
  against the new signatures in this batch
- the ceiling is `MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES`, `268435456` samples, and
  it counts every channel rather than frames
- ratio `1.0e6` over `4096` frames now returns
  `OutputTooLarge { requested_samples: 4096000000, .. }` instead of allocating
  for roughly a minute
- four `A1` and `A4` owners activated; `A2` and `A3` remain ignored for Batch
  36.4

Byte-exactness proof and one method correction. The first attempt froze output
hashes for ratios `0.5..3.0` in an integration test. Those hashes passed under
`--release` and failed under the default profile: f32 render output is
optimization-profile dependent, so an absolute hash is only valid in the
profile that captured it. The claim is now proven structurally instead, in
`phase_vocoder::tests`:

- `overlap_safe_analysis_hop_is_a_no_op_through_ratio_three` asserts the law
  returns the configured `512` hop for every ratio through `3.0` at the
  retained `2048/512` geometry, so that path is unchanged by construction
- `overlap_safe_analysis_hop_bounds_the_synthesis_hop` asserts the adapted
  hops and that each keeps synthesis hop inside the bound
- the existing `phase_vocoder_bit_exact_baseline` at ratio `1.5` still passes
  with its original hash `0x8255b18311f778f9`, unchanged

Release-profile hashes captured before and after the change, recorded as
Contract `084` Rule 10 evidence:

| ratio | mono before | mono after | stereo before | stereo after |
| --- | --- | --- | --- | --- |
| `0.5` | `0xdce5d9954045670d` | unchanged | `0x03c96dadebe4325c` | unchanged |
| `0.75` | `0x9c6cb46cdc2d4daa` | unchanged | `0xa1fc4f0fad525e2a` | unchanged |
| `1.25` | `0xa7bd73eb6ebc5adb` | unchanged | `0x25f97bf0ae8db5d2` | unchanged |
| `1.5` | `0x9a09a840aa6cd8d2` | unchanged | `0x4ee821fc49823f7b` | unchanged |
| `2.0` | `0xcd8a6315902c52af` | unchanged | `0xa1d9a0fb721bb897` | unchanged |
| `2.5` | `0xcaf4c3bba062b56d` | unchanged | `0x9a499af8216f58be` | unchanged |
| `3.0` | `0xead24515a648ae83` | unchanged | `0xcf279977c20cc666` | unchanged |
| `4.0` | `0xd3e45b0e45a3d728` | `0x16322f772f1ec017` | `0x94dfeede7146ce2d` | `0xdba1fa57b58acaf3` |
| `6.0` | `0x9b0c8257654a48d8` | `0xbbcc28475460355e` | `0xaf7195637006766d` | `0xb88dbc7e3d98cac9` |

Render cost, one second of 48 kHz mono, release profile:

| ratio | before | after |
| --- | --- | --- |
| `2.0` | `5.08 ms` | `5.08 ms` |
| `3.0` | `5.01 ms` | `5.01 ms` |
| `4.0` | `5.1 ms` | `7.98 ms` |
| `6.0` | `5.2 ms` | `11.86 ms` |
| `8.0` | `5.3 ms` | `19.44 ms` |

Cost rises only where the hop tightens, which is only where the old renderer
was producing silence or ripple.

### Batch 36.4 - Dynamic Ratio Segment And Seam Correction

Status: complete; admitted under Contract `084` Rule 5 by explicit operator decision

- [x] coalesce or resample dynamic-ratio segments so no admitted curve produces
  a sub-window segment
- [x] give the mono dynamic-ratio path the same seam treatment as linked stereo
- [x] measure dense-curve pitch error and mono/stereo seam click before and
  after
- [ ] run concealed listening on dynamic-ratio material before admission
- [x] record the re-baselined byte-exact hashes with the evidence that
  justified each change

Result:

- `coalesce_short_dynamic_ratio_segments` merges adjacent spans until every
  segment carries at least `min_dynamic_ratio_segment_frames`, with the merged
  target equal to the sum of its constituent targets so total output length and
  average tempo are preserved exactly
- the frozen minimum is `window_size + 8 * analysis_hop`, `6144` source frames
  at the retained geometry. Contract `046` had frozen one window; measurement
  showed one window leaves `19.6` cents of pitch error because a single-window
  segment gives the phase vocoder one analysis frame. Eight extra hops leave
  `2.8` cents. Longer minimums buy nothing audible and cost ratio-curve time
  resolution. Contract `046` is amended with the sweep and the chosen value
- the mono path now runs the same seam pass the interleaved path ran

| measurement | before | after |
| --- | --- | --- |
| dense-curve dominant frequency | `220.0 Hz` | `440.7 Hz` |
| dense-curve output length | `96000` | `96000` |
| mono seam click | `-28.940011 dBFS` | `-180.617997 dBFS` |
| stereo seam click | `-180.617997 dBFS` | `-180.617997 dBFS` |
| tempo-ramp dominant frequency | — | `440.5 Hz` |

All ten owners are active and passing; none remain ignored. The dense-curve
pitch tolerance tightened from `2%` to `0.5%` against a measured `0.16%`, and
`segment_coalescing_preserves_total_output_length` is new, covering four curve
shapes in mono and linked stereo.

Scope boundary: `plan_offline_stretch_chunks` still segments on raw curve
points, so the artifact path can still produce sub-window chunks. `g10.039`
owns that, because the durable fix is a renderer that carries state across a
chunk rather than a longer minimum.

Two concealed listening rounds ran. Round one: `C5` confirmed the overlap
correction, `C2` and `C3` tied, and `C1` rejected the corrected side for a
secondary rhythmic pulse. Round two raised the segment minimum from
`window + 8 hops` to `window + 32 hops`, cutting measured envelope modulation
from `0.545 dB` to `0.115 dB` and making dense-curve pitch exact. The operator
still heard the pulse, on a different time base.

That change of time base identified the mechanism. Rendering a constant ratio
through the segmented path against the same ratio rendered whole gives
correlation `0.034`, peak sample difference `1.1470`, and a difference RMS of
`0.2474` against a signal RMS of `0.1784`. Each segment restarts the phase
vocoder, so phase across every join is arbitrary. The artifact is phase
re-initialisation, not amplitude modulation, and segment length only changes
its rate. Contract `084` Rule 7 closes segmentation tuning as a mechanism.

This predates the lane. Contract `046` already required continuous algorithm
state or a measured transition rather than raw segment concatenation, and the
implementation has never satisfied it.

Admitted under Rule 5 by explicit operator decision rather than a clean pass:
the correction replaces an octave-wide pitch error with a milder seam artifact,
takes dense-curve pitch from `220.0 Hz` to `440.0 Hz`, and cuts joins from
about `46` to `10`. The residual is recorded with its measurement.
`segmented_render_matches_whole_render_at_constant_ratio` is `#[ignore]`d and
holds `g10.039`'s acceptance target of `0.99` correlation.

`A18`, low-end pops on transients found in both sides of round one, predates
this lane and is almost certainly the same mechanism at transient positions. It
needs triage before routing.

### Batch 36.5 - Re-Baseline And Closeout

Status: complete

- [x] run the full corpus comparison and acceptance report
- [x] run `effigy validate` and the full crate suite
- [x] update Contract `046`, Contract `084`, and the `g10` front doors to the
  corrected behavior
- [x] publish the corrected Transparent behavior matrix
- [x] name the next ready batch in `g10.037`

Corpus comparison: `27` comparisons, `14` improved, `0` regressed, `13`
unchanged, `0` inconclusive. Five missing required assets, all operator-provided
licensed listening families held outside the repository by the source policy.

## Corrected Transparent Behavior

| behavior | before `g10.036` | after |
| --- | --- | --- |
| overlap coverage above ratio `4.0` | interior blocks zeroed | full coverage at every accepted ratio |
| ripple at ratio `4.0` | `1.396 dB` | `0.276 dB` |
| ratios `0.5x..3.0x` | — | byte-identical, proven structurally |
| dense ratio curve | `220.0 Hz` from a `440 Hz` source | `440.0 Hz` |
| dynamic-ratio segment minimum | none | `window + 32 hops`, `384 ms` at 48 kHz |
| sub-minimum curve spans | rendered as varispeed | merged at the mean ratio, length exact |
| mono seam click | `-28.940011 dBFS` | `-180.617997 dBFS`, matching stereo |
| oversized render | allocated `4096000000` samples | refused with `OutputTooLarge` |
| whole-buffer ceiling | none | `268435456` output samples |
| segment join phase | arbitrary, unmeasured | arbitrary, measured and recorded; `g10.039` owns it |

## Acceptance Criteria

- [ ] no interior output block is zeroed at any ratio the API accepts
- [ ] every admitted dynamic-ratio curve preserves pitch within the frozen
  Transparent tolerance
- [ ] mono and linked-stereo seam click agree within one frozen tolerance
- [ ] output allocation is bounded and over-large requests are refused, not
  attempted
- [ ] the full crate suite passes twice with identical results
- [ ] byte-exact output over `0.5x..3.0x` for every extension-class correction
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

Execute `g10.037` Batch 37.1: enumerate every input that changes rendered
output against the current cache identity fields, amend Contract `046` for
render geometry and stable key tokens, and decide the schema advance and the
creative-cache position. Documentation only.

`A18` needs triage before routing. The working hypothesis is that `g10.039`
resolves it with the seam pulse, since both trace to segment phase restarts.
