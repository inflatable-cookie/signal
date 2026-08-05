# 040 - RealtimePreview Completion

Status: active; brief frozen by Batch 40.2; Batch 40.3 ready
Owner: dsp
Created: 2026-07-27
Depends on: `g10.036`, `g10.038`, `g10.039`
Supersedes: `g10.024`, `g10.028`
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
`docs/contracts/084-stretch-candidate-isolation-and-promotion-contract.md`
Vision tags: `DSP`, `STRETCH`, `REALTIME`, `PREVIEW`

## Problem

RealtimePreview is the one stretch tier Signal wants and does not have. Live
pitch-preserving preview is genuinely useful; the current implementation is not
usable and has stalled across three roadmaps.

What exists: `g10.026` proved callback-local allocation-free STFT work, and
`g10.027` proved source-projection reporting and dynamic-ratio source/output
continuity. `g10.024` and `g10.028` are paused. Contract `046`'s callback gate
addendum holds `audio_thread_processing_supported` at `false` while the stream
is `QuantumLocked`.

Why it stalled, from the 2026-07-27 audit. `RealtimePreviewCallbackState::process`
is quantum-locked: it consumes `n` input frames and produces `n` output frames
regardless of ratio. A time stretcher cannot be rate-locked on both sides. At
ratio above `1.0` the synthesis cursor outruns the output ring, the analysis
loop breaks on its ring guard, input backs up, and the guard then jumps
`next_analysis_frame` forward — dropping source audio. `process` returns `Ok`
while doing this. The projection machinery added in `g10.027` reports the
correct source advance but nothing consumes it, so the report and the kernel
disagree by construction.

Audit finding `A11` records what accumulated around that gap: roughly `1100`
lines in `lib.rs`, a state object with about `45` fields, around `30` trivial
getters, two parallel ratio schedulers that duplicate each other, and six enum
variants that are never constructed anywhere in the workspace —
`IntegrationMode::CallbackSafeStreaming`,
`CallbackTimelineMode::SourceProjected`,
`UnsupportedMode::AudioThreadProcessing`,
`UnsupportedMode::SourceAdvanceContract`,
`UnsupportedMode::ChannelLayout`, and
`CallbackProcessError::CallbackProcessingUnsupported`. No workspace consumer
imports any `RealtimePreview` type. `fft_plans_ready` returned a condition that
can never be false and was removed by `g10.038`.

The missing piece is not more reporting. It is source ownership: the callback
must pull the source frames its ratio demands, own its own fill, underrun, and
latency behavior, and stop pretending input and output advance at the same
rate.

## Generation Runway

This lane is deliberately last in the audit-driven sequence. It is the only one
that needs a new design rather than a correction, and it starts from a crate
that `g10.036` through `g10.039` have already made correct, identified, small,
and resumable.

The visible runway is:

1. feasibility and design reassessment — is callback-safe streaming reachable
   with the current kernel, and at what latency
2. one complete streaming brief, or an explicit closure
3. one isolated implementation
4. render-plane integration behind the existing gate
5. surface reduction and closeout

The next planning checkpoint is Batch 40.1. It may close the tier instead of
opening a brief; that is a valid and recorded outcome, and it would then delete
the unreachable surface rather than carry it.

## Goals

- [ ] decide honestly whether callback-safe pitch-preserving preview is
  reachable with the admitted kernel
- [ ] if reachable, give the callback path ownership of source fill, input
  demand, underrun, and latency
- [ ] make reported source projection and actual kernel consumption the same
  number
- [ ] reach `CallbackSafeStreaming` and `SourceProjected` honestly, or record
  why not and remove the surface that implies them
- [ ] leave one tier state that matches shipped behavior

## Non-Goals

- no change to the Transparent offline renderer or its output
- no creative-character work
- no new offline algorithm family
- no cache, artifact, or export change
- no Loophole or Chorus surface
- no promotion of preview quality claims without the Contract `046` evidence

## Execution Plan

### Batch 40.1 - Feasibility And Design Reassessment

Status: complete; decided **reachable**

Documentation only.

- [x] state the quantum-locked defect exactly, with the input-drop path traced
- [x] measure achievable algorithmic latency for the `512/128` preview geometry
  and state what it costs against `MAX_BLOCK_FRAMES` and the render quantum
- [x] design the source-ownership model: who holds the source reader, how the
  callback expresses input demand, what happens on underrun, and how latency is
  reported
- [x] decide whether the render plane pulls preview output or the preview state
  pulls source frames
- [x] check the model against Contract `046`'s callback gate: bounded work, no
  allocation, no locks, no I/O, deterministic latency, linked stereo,
  dynamic-ratio alignment, seam evidence
- [x] decide reachable or not reachable, and record the reason either way
- [x] change documentation only

Stop condition: if the model requires I/O or unbounded work on the callback,
record that RealtimePreview cannot be a callback tier with this kernel, close
the tier, and route Batch 40.5 to surface removal instead of integration.

The stop condition is not met. Neither I/O nor unbounded work is required, and
the tier stays open.

#### The Defect, Traced

`process` is quantum-locked in two lines:

```rust
self.push_interleaved_input(input, frame_count);
self.process_available_streaming_frames();
self.read_interleaved_output(output, frame_count);
```

`frame_count` in, `frame_count` out, whatever the ratio. A time stretcher
cannot be rate-locked on both sides, so the two cursors diverge by
construction: `next_analysis_frame` advances `analysis_hop` per spectral frame
while `next_synthesis_frame` advances `analysis_hop * ratio`.

The drop follows in `process_available_streaming_frames`. At ratio above `1.0`
synthesis outruns the output ring, so this guard breaks the loop:

```rust
if synthesis_start + window_size >= output_read_frame + ring_frames { break; }
```

Nothing is analysed that callback, but `push_interleaved_input` keeps
advancing `input_write_frame` by `frame_count` every callback. The gap widens
until this fires:

```rust
if input_write_frame - next_analysis_frame > ring_frames {
    next_analysis_frame = input_write_frame - ring_frames;
}
```

That assignment discards every unanalysed source frame in the gap. `process`
then returns `Ok` with `input_frames == output_frames == frame_count`, so the
caller is told the block was consumed and produced normally. The
`g10.027` projection machinery reports the correct source advance beside this,
which is why the report and the kernel disagree: nothing consumes the report.

#### Cost, Measured

Steady-state cost of the existing callback path at ratio `1.0`, `48 kHz`,
release build, `4000` iterations after a `64`-callback warmup:

| channels | block | per callback | per spectral frame | budget | load |
| --- | --- | --- | --- | --- | --- |
| `1` | `128` | `8.0us` | `8.0us` | `2666.7us` | `0.3%` |
| `1` | `512` | `31.8us` | `8.0us` | `10666.7us` | `0.3%` |
| `2` | `128` | `15.6us` | `15.6us` | `2666.7us` | `0.6%` |
| `2` | `512` | `62.7us` | `15.7us` | `10666.7us` | `0.6%` |

Cost is linear in spectral frames and flat per frame across block sizes, so it
projects cleanly. A callback producing `block` output frames needs
`block / (analysis_hop * ratio)` spectral frames, so load scales as `1/ratio`.
Stereo at block `128`:

| ratio | spectral frames | callback cost | load |
| --- | --- | --- | --- |
| `1.0` | `1` | `15.7us` | `0.6%` |
| `0.5` | `2` | `31.4us` | `1.2%` |
| `0.25` | `4` | `62.8us` | `2.4%` |
| `0.125` | `8` | `125.6us` | `4.7%` |
| `0.0625` | `16` | `251.2us` | `9.4%` |

**CPU was never the blocker.** Even at one-sixteenth speed the callback uses
under a tenth of its budget. What stalled this tier was the architecture, not
the kernel's cost, and three roadmaps of latency and reporting work were spent
without that number being measured.

#### Bounded Work Needs A Bounded Ratio

`sanitize_ratio` accepts any finite positive ratio. Work per callback scales as
`1/ratio`, so as ratio approaches zero the callback work is unbounded and
Contract `046`'s bounded-work requirement is unsatisfiable as written. This is
not a tuning detail; it is a gate precondition. Batch 40.2 must freeze a
minimum ratio, and the table above is the evidence for choosing it.

#### Latency

Algorithmic latency is one window: `512` frames, `10.67ms` at `48 kHz`. A full
window must be buffered before the first synthesis frame exists. Against
`MAX_BLOCK_FRAMES` of `4096` (`85ms`) it is small; against a `128`-frame
quantum (`2.67ms`) it is four quanta.

This is affordable because of what preview *is*. RealtimePreview plays back a
stored asset at a changed rate — it is not a live monitoring path, so the
window latency is a start-up delay before playback begins, not a round-trip
cost added to something the operator is playing. Ten milliseconds of start-up
is not perceptible as latency.

#### Source Ownership

The render plane pulls preview output; the preview state pulls source frames.
Both, at different boundaries — that is the answer to the batch's question,
and treating it as one choice is what made it look unresolvable.

- A non-realtime producer owns the source reader and fills a single-producer,
  single-consumer ring with source frames. All I/O happens there.
- The callback consumes `block / ratio` source frames per callback from that
  ring. It never reads a file, takes a lock, or allocates.
- Input demand is published as an atomic frame counter the producer reads: the
  callback states how far ahead it needs the source filled, derived from the
  ratio it is about to apply. The `g10.027` projection already computes this
  number correctly; the change is that something finally consumes it.
- On underrun the callback emits silence for the missing span, increments an
  underrun counter, and returns a report saying so. It must not stall, must not
  skip source, and must not return `Ok` as though the block were normal — that
  last behaviour is the present defect.
- Latency is reported as window size plus the producer's prefill target, both
  constant for a given configuration.

#### Contract 046 Callback Gate

| requirement | status |
| --- | --- |
| bounded work | conditional — requires the frozen minimum ratio above |
| no allocation | already proven; `capture_alloc`-style test covers the path |
| no locks | satisfied by the SPSC ring and atomic demand counter |
| no I/O | satisfied; the producer thread owns the reader |
| deterministic latency | satisfied; window plus prefill, both constant |
| linked stereo | already supported by the kernel |
| dynamic-ratio alignment | machinery exists from `g10.027`, unconsumed |
| seam evidence | not yet produced; Batch 40.2 must freeze it |

#### A Naming Hazard Worth Recording

The Problem section says no workspace consumer imports any `RealtimePreview`
type. That is true of the callback surface and false of the name.

`RealtimePreviewStretcher` is a whole-buffer control-side prototype with a
shorter window — not a callback object at all — and `loophole/pulse` consumes
it in `render_plan.rs` to pre-stretch and cache assets. Closing the tier and
deleting "the RealtimePreview surface" wholesale would have broken a shipping
consumer.

What is genuinely unconsumed is `RealtimePreviewCallbackState` and the six
never-constructed enum variants named in the Problem section. Batch 40.5 must
name types, not the prefix.

### Batch 40.2 - Complete Streaming Brief

Status: complete

Documentation only.

- [x] freeze the minimum ratio, without which bounded work is unsatisfiable
- [x] freeze the callback state inventory, its capacities, and its memory
  ceiling
- [x] freeze the source fill, input demand, and underrun policy
- [x] freeze one ratio scheduler; the current two are redundant
- [x] freeze the latency report and its relationship to the stream contract
- [x] freeze dynamic-ratio alignment tolerance and its seam evidence
- [x] freeze the evidence order, rejection rules, and cleanup
- [x] change documentation only

Everything below is frozen. Batch 40.3 implements it once and does not
renegotiate it; a defect in the brief is corrected here and re-frozen, not
patched in the implementation.

#### Frozen: Ratio Range `[0.25, 3.0]`

Both ends are derived, not chosen.

The maximum is Contract `046`'s overlap law: `analysis_hop * ratio <= 0.75 *
window_size`. At the frozen `128`/`512` geometry that is exactly `3.0`. Going
beyond it requires the contract's hop reduction, which changes the geometry and
therefore the state inventory, so it is out of scope for this brief. Higher
ratios are cheap — `0.20%` at ratio `3.0` — so the limit is coverage, not cost.

The minimum is bounded work. Load scales as `1/ratio`, measured in Batch 40.1:

| ratio | stereo load, `128`-frame callback |
| --- | --- |
| `3.0` | `0.20%` |
| `1.0` | `0.59%` |
| `0.5` | `1.18%` |
| `0.25` | `2.36%` |
| `0.125` | `4.72%` |

`0.25` is four-times-faster playback at `2.36%` of budget. Widening to `0.125`
costs `4.72%` and doubles the source ring; the headroom exists, so this is a
product decision rather than an engineering limit, and it can be revisited
without touching the design. What cannot be revisited is having no floor at
all: `sanitize_ratio` currently accepts any positive value, which makes bounded
work unsatisfiable.

Ratios outside the range are rejected at plan time, not clamped silently.

#### Frozen: State Inventory And Memory

Measured, stereo, with an allocation-counting global allocator:

| block | current state |
| --- | --- |
| `128` | `141.3 KiB` |
| `512` | `180.3 KiB` |
| `4096` (`MAX_BLOCK_FRAMES`) | `544.3 KiB` |

The streaming model adds one source ring, sized
`ceil(block / ratio_min) * 2 + window_size` frames — two callbacks of headroom
plus one window:

| block | source ring at `ratio_min = 0.25` |
| --- | --- |
| `128` | `12.0 KiB` |
| `512` | `36.0 KiB` |
| `4096` | `260.0 KiB` |

Ceiling: **`1 MiB` stereo at `MAX_BLOCK_FRAMES`**, against a computed
`544.3 + 260.0 = 804.3 KiB`. The margin is deliberate but the number is derived
from the design rather than preceding it — `g10.039` moved its ceiling three
times by freezing one before the design existed, and Contract `046` records
why.

Field count drops by `11`: the duplicate scheduler below is deleted. It rises
by the source ring, its write cursor, and its demand counter. Batch 40.3 must
state the final count and it must not exceed the current `56`.

#### Frozen: One Ratio Scheduler

The state carries two, field for field:

```
current_ratio                              source_projection_current_ratio
active_ratio                               source_projection_active_ratio
pending_ratio                              source_projection_pending_ratio
pending_ratio_request_frame                source_projection_pending_ratio_request_frame
pending_ratio_apply_frame                  source_projection_pending_ratio_apply_frame
pending_ratio_change                       source_projection_pending_ratio_change
last_ratio_change_request_frame            last_source_projection_ratio_change_request_frame
last_ratio_change_output_frame             last_source_projection_ratio_change_output_frame
last_ratio_change_alignment_error_frames   last_source_projection_ratio_change_alignment_error_frames
ratio_change_count                         source_projection_ratio_change_count
last_ratio_change_applied_frame            last_source_projection_ratio_change_source_frame
```

The **source-projection scheduler survives**; the output-side one is deleted.
That is the direction that looks backwards and is not: the projection scheduler
already computes the source advance a ratio implies, which is precisely the
number the new model needs to drive source demand. The output-side scheduler
computes synthesis advance, which the quantum-locked kernel needed only because
it could not ask for source.

The `g10.027` projection machinery was never wrong. Nothing consumed it. This
brief makes it the single authority.

#### Frozen: Source Fill, Input Demand, Underrun

- A non-realtime producer owns the source reader and fills a single-producer,
  single-consumer ring. All I/O lives there. The callback never opens, reads,
  seeks, locks, or allocates.
- Per callback the kernel consumes `block / active_ratio` source frames,
  fractional, with the fractional part carried in the existing source cursor.
- Demand is published as one atomic frame counter: the source frame index the
  callback needs filled to. The producer reads it and fills ahead. This is the
  projection's output, now consumed.
- Prefill target is `ceil(block / ratio_min) * 2 + window_size` frames — the
  ring capacity above. Playback does not start until it is met.
- On underrun the callback emits silence for the missing span, increments an
  underrun counter, and returns a report naming the shortfall in frames. It
  must not stall, must not advance past unfilled source, and must not return a
  report indistinguishable from a normal block. That last behaviour is the
  present defect and is what let the quantum lock hide for three roadmaps.

#### Frozen: Latency Report

Reported latency is `window_size + prefill_frames`, constant for a given
configuration and computed at plan time. At the frozen geometry with
`ratio_min = 0.25` and block `512`: `512 + 2560 = 3072` frames, `64ms` at
`48 kHz`.

This is a start-up delay before preview playback begins, not a round-trip cost
on a live signal, because preview plays back a stored asset. The stream
contract reports it as such and `audio_thread_processing_supported` may only go
`true` once it is honoured.

#### Frozen: Dynamic-Ratio Alignment And Seam Evidence

Ratio changes align to the analysis-hop grid via the existing
`align_to_next_grid`, so alignment error is bounded by `analysis_hop` — `128`
frames — by construction. **Tolerance: `<= 128` output frames.** A change
reporting more than that is a defect, not a tuning miss.

Seam evidence, one gate: a sustained source rendered with a ratio change
mid-stream must correlate with a whole-buffer control at the same ratio curve
above the threshold Contract `046` already freezes for the offline path. The
`g10.036` seam pulse is the failure this catches.

#### Frozen: Evidence Order, Rejection, Cleanup

Order, each blocking the next:

1. allocation-free, lock-free, I/O-free callback execution, proven by an
   allocation-counting harness like the existing preview callback test
2. bounded work across the frozen ratio range, measured, not asserted
3. sustained ratios above and below `1.0` produce continuous output with no
   dropped source — the defect Batch 40.1 traced
4. underrun produces reported silence rather than a normal-looking block
5. dynamic-ratio alignment within `128` frames
6. seam correlation against the whole-buffer control

Rejection: any of the six failing rejects the candidate under Contract `084`
Rule 2 and it is reverted from `main`, not iterated in place. Structural
conformance may iterate per Rule 11; the acoustic checkpoint is one-shot.

Cleanup: a rejected candidate leaves no partial surface. `RealtimePreviewStretcher`
is out of scope in both directions — it is consumed by `loophole/pulse` and must
not be touched by this lane.

### Batch 40.3 - Isolated Implementation

Status: ready; Batch 40.2 froze the brief on 2026-08-05

- [ ] implement the frozen brief once, isolated per Contract `084` Rule 2
- [ ] prove allocation-free, lock-free, I/O-free callback execution
- [ ] prove sustained ratios above and below `1.0` produce continuous output
  with no dropped source frames
- [ ] prove reported source consumption equals actual kernel consumption
- [ ] prove dynamic-ratio changes land inside the frozen alignment tolerance
- [ ] measure preview quality against the offline renderer on the same material

### Batch 40.4 - Render Plane Integration

Status: blocked on Batch 40.3

- [ ] open `CallbackSafeStreaming` and `SourceProjected` only after Batch 40.3
  passes every gate together
- [ ] set `audio_thread_processing_supported` from proven behavior, not from a
  constant
- [ ] integrate behind the existing render-plane preview boundary
- [ ] prove no callback deadline miss under soak

### Batch 40.5 - Surface Reduction And Closeout

Status: blocked on Batch 40.4, or on a Batch 40.1 closure decision

- [ ] remove every enum variant, getter, and scheduler the shipped design does
  not use, including the six never-constructed variants named above
- [ ] if Batch 40.1 closed the tier, remove the whole unreachable contract
  surface and leave one honest statement of what Signal does not have
- [ ] mark `g10.024` and `g10.028` resolved through this lane
- [ ] run `effigy validate` and the full crate suite
- [ ] update Contract `046`, the stretch boundary, and the `g10` front doors

## Acceptance Criteria

- [ ] the quantum-locked defect is fixed or the tier is explicitly closed
- [ ] no path drops source frames while returning success
- [ ] reported and actual source consumption agree, proven
- [ ] callback execution is allocation-free, lock-free, and I/O-free, proven
- [ ] `audio_thread_processing_supported` reflects proven behavior
- [ ] no never-constructed variant or unreachable contract surface remains
- [ ] `g10.024` and `g10.028` are resolved rather than left paused
- [ ] full crate suite passes

## Risks and Mitigations

- Risk: another partial pass leaves a fourth stalled RealtimePreview roadmap.
  Mitigation: Batch 40.1 may close the tier, and closure routes directly to
  surface removal. A recorded no is a successful outcome for this lane.
- Risk: source fill pulls I/O onto the callback. Mitigation: it is an explicit
  Batch 40.1 stop condition.
- Risk: preview quality is unusable even when the mechanism is correct.
  Mitigation: Batch 40.3 measures preview against the offline renderer before
  integration opens.
- Risk: the surface grows again while the design is unproven. Mitigation: Batch
  40.2 freezes the state inventory before implementation and one scheduler
  replaces two.

## Evidence Requirements

- [ ] one log per completed batch under `docs/logs/`
- [ ] the Batch 40.1 latency measurement and reachability decision
- [ ] the frozen state inventory and memory ceiling
- [ ] sustained-ratio continuity proof with no dropped source frames
- [ ] allocation, lock, and I/O freedom proof on the callback path
- [ ] soak evidence for callback deadlines before integration
- [ ] commands actually run

## Next Task

Open Batch 40.3, the isolated implementation. Batch 40.2 froze the brief and it
is not renegotiable in the implementation: a defect in the brief is corrected in
`40.2` and re-frozen, per Contract `084`.

What is frozen: ratio range `[0.25, 3.0]`, both ends derived — the maximum is
the overlap law at the `128`/`512` geometry, the minimum is bounded work at
`2.36%` of a stereo `128`-frame callback. A `1 MiB` stereo ceiling at
`MAX_BLOCK_FRAMES` against a computed `804.3 KiB`. One ratio scheduler, the
source-projection one, with the output-side duplicate deleted. Source fill by a
non-realtime producer through an SPSC ring, demand published as one atomic,
underrun reported as silence rather than as a normal block. Latency
`window_size + prefill`, `3072` frames at block `512`. Alignment tolerance
`128` output frames. Six pieces of evidence in a blocking order.

The implementation must land isolated per Rule 2, and must not touch
`RealtimePreviewStretcher`, which `loophole/pulse` consumes.

Also inherited from `g10.039` and still open: adopting the remaining offline
paths so both seam smoothers can be removed, and a direct transient probe for
`A18`.
