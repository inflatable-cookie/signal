# g10.040 Batch 40.4 - Tier Opened, Integration Re-Scoped

Status: complete for what was real; integration item re-scoped to Batch 40.6
Created: 2026-08-05
Scope: RealtimePreview callback tier flags, deadline soak, render-plane boundary

## The Tier Opens, From Properties Rather Than Assertion

`RealtimePreviewStreamState::contract` now reports `CallbackSafeStreaming` and
`SourceProjected` with `audio_thread_processing_supported` true. Each flag is
carried by a specific gate: allocation-free (`G1`); bounded work, which holds
only because ratios outside `[0.25, 3.0]` are rejected rather than clamped
(`G2`); source consumption tracking the ratio with nothing dropped (`G3`, `G4`);
starvation reported rather than hidden (`G5`); changes landing within one
analysis hop (`G6`).

The checklist asked for this to come from proven behaviour rather than a
constant, and the distinction is real. The gates prove properties of one kernel
at one geometry, so the contract re-checks the envelope those proofs assume —
channel count, block size, the overlap law bounding the maximum ratio, and the
working-set ceiling — and reports unsupported outside it. `G8` additionally pins
the reported alignment tolerance to the value `G6` proves, so the contract
cannot advertise a looser number than the evidence supports.

The shipped `RealtimePreviewCallbackState` continues to report `QuantumLocked`
and unsupported, because it still is both.

## Measured

Working set at the widest supported configuration — stereo at
`MAX_BLOCK_FRAMES` — is `404564` bytes, `395.1 KiB`, against the frozen `1 MiB`
ceiling.

Batch 40.2 predicted `804.3 KiB`, so the estimate overshot by roughly `2x`. It
added the source ring to the *shipped* kernel's measured allocation, and the new
kernel's own state is leaner. Recording the gap rather than quietly banking the
margin: an estimate that overshoots by `2x` is lucky, not good, and the next one
should be built from the design's own buffers.

`G9`, in the soak lane because it is a wall-clock claim about the host:
`20000` callbacks sweeping the whole frozen ratio range, worst callback
`208.8us` against a `2666.7us` budget — `7.8%` of the deadline — with `0`
misses.

## The Boundary Named In The Checklist Does Not Exist

Batch 40.4 was to "integrate behind the existing render-plane preview boundary".
There is no such boundary.

`signal-render-plane/src/lib.rs` contains zero occurrences of "preview". Every
reference in the crate lives in `offline.rs` and exists to *reject*
`StretchBackendTier::RealtimePreview` from offline artifact planning. The
preview tier has never been wired for live rendering.

This is the second premise in this roadmap that did not survive contact with the
code. Batch 40.1 found that `RealtimePreviewStretcher` is consumed by
`loophole/pulse` and is not dead surface despite the shared prefix; this one
assumed an integration point that was never built.

Both were written into the roadmap in good faith at planning time, and both
would have been carried forward as fact if the batch had started from the
checklist rather than from the code.

## Why It Is Re-Scoped Rather Than Built

Building the path is a new design, not an integration: the producer thread that
fills the source ring, its I/O, the transport and seek behaviour that implies,
and how a preview stage sits in the render graph.

Nothing needs it. `loophole/pulse` pre-stretches whole buffers and caches them,
which is what the whole-buffer prototype is for, and no other consumer
references the preview tier. Building a live callback path speculatively is the
over-engineering the audit that opened this generation was about — and it is how
the surface Batch 40.5 now has to delete came to exist in the first place.

It is Batch 40.6, gated on a consumer asking. The kernel is proven and waiting.

## Admission

Nothing in this batch changed shipped audio. The streaming kernel is still
constructed by nothing outside its own tests, so Contract `084` Rule 2 isolation
holds and Rule 5 admission has not been sought.

## Next Task

Open Batch 40.5, surface reduction and closeout.
