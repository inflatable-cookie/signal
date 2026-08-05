# g10.040 Batch 40.5 - Surface Reduction And Closeout

Status: complete
Created: 2026-08-05
Scope: dead surface removal, superseded roadmaps, contract amendment

## Two Of The Six Variants Became Real

The roadmap named six never-constructed enum variants as deletion candidates.
Only four still qualify. `IntegrationMode::CallbackSafeStreaming` and
`CallbackTimelineMode::SourceProjected` are constructed by
`RealtimePreviewStreamState::contract` as of Batch 40.4.

The roadmap allowed either outcome — make them true or delete them. This lane
made them true, which was always the better one.

Removed, each with no construction site anywhere in the workspace:

- `RealtimePreviewUnsupportedMode::AudioThreadProcessing`
- `RealtimePreviewUnsupportedMode::SourceAdvanceContract`
- `RealtimePreviewUnsupportedMode::ChannelLayout`
- `RealtimePreviewCallbackProcessError::CallbackProcessingUnsupported`

`SourceBufferingContract` stays. `plan_realtime_preview_stream` constructs it and
it is the honest statement of what the shipped kernel does not do.

## The Getters Were Already Gone

`A11` recorded roughly `30` trivial getters. Measured: `realtime_preview.rs`
exposes `36` public functions, of which exactly `1` is never called anywhere —
`last_source_projection_ratio_change_request_frame`. `g10.038` had already taken
the rest when it cut `lib.rs` from `5181` to `3343` lines.

Worth stating plainly because the checklist implied `30` deletions were waiting.
Carrying a finding forward without re-measuring it would have produced a batch
that reported removing surface an earlier lane had already removed.

## Test-Only Surface Is Not Dead Surface

`27` of the `36` public functions are reachable only from tests. That is real and
worth naming, and it is not a deletion list.

Most of it exists to assert bounded working set and ratio-change alignment,
which Contract `046` requires proving. Deleting it deletes the proof that the
*currently shipped* kernel meets those requirements — and the replacement is not
admitted, because admission needs listening under Rule 5.

So it leaves with the kernel it introspects rather than before it. The same
applies to the duplicate ratio scheduler Batch 40.2 identified: it lives in the
shipped kernel, which needs both halves to work. The new kernel already has one.

Deleting a shipped component's test surface to make a surface-reduction batch
look finished would be scoring the metric instead of doing the work.

## Superseded Roadmaps

`g10.024` and `g10.028` move from `paused` to resolved through this lane.
`g10.024` asked whether RealtimePreview could be a real callback tier — answered
reachable, implemented, proven. `g10.028` was to define the source fill contract
— frozen in Batch 40.2 and implemented in Batch 40.3.

## Contract 046

The `2026-07-09` callback gate addendum is satisfied, and the amendment records
that it is satisfied by a different kernel than the one it was written against.
The original could never have passed it: with no way to ask for source, owning
"source-buffer fill or underrun behavior" was unmeetable by construction.

The amendment also carries forward the two limits that are part of the gate
rather than exceptions to it — bounded work requires a bounded ratio, and a
kernel that clamps an out-of-range ratio rather than rejecting it does not
satisfy the gate, because a silent clamp makes reported and actual source
advance disagree.

## Lane State

`g10.040` is complete. The callback tier is reachable, implemented, and proven
by nine gates. It is not adopted: nothing outside its tests constructs it, Rule 2
isolation holds, and Rule 5 admission has not been sought.

Batch 40.6, a live render-plane preview path, is open and gated on a consumer
asking rather than on this roadmap's assumption.

## Next Task

Nothing in `g10.040`. Open items elsewhere: adopting the remaining offline paths
so both seam smoothers can be removed, and a direct transient probe for `A18`.
