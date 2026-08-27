# System Architecture

Status: active
Owner: core-product
Updated: 2026-08-17
Vision refs: `docs/vision/001-signal-vision.md`

## Top-Level Stack

Signal is the shared audio-systems stack used by Loophole, Finch, and future
apps. Its layers, from the bottom up:

1. `signal-primitives`
   - sample/frame/time/channel types, buffer primitives, realtime-safe
     utility types
2. `signal-dsp`
   - reusable DSP kernels and transforms: smoothing, metering, resampling,
     filters
3. `signal-analysis`
   - onset, beat, tempo, tonal, loudness, and character analysis, reusable
     offline and streaming
4. `signal-graph`
   - graph plan model and contract summaries for the control plane
5. `signal-runtime`
   - thin, embeddable control library: lifecycle, graph plan vocabulary,
     plugin discovery/sandbox records, media pipeline, observation reports
6. host-edge adapters
   - plugin-format adapters (CLAP/VST3/AU/LV2 discovery) and hardware/device
     adapters, kept narrow
   - plugin processing backends (in-process and sandboxed) behind the
     render-plane plugin handle

The authoritative crate inventory is `docs/architecture/system-inventory.md`.
The older `package-map.md` naming proposal is superseded in part.

The realtime audio path itself lives in `signal-render-plane`:
precompiled, alloc-free plan execution with declick envelopes and polyphase
clip resampling, driven from negotiated device streams via `signal-hardware`.

## Data and Authority Flow

- Signal owns audio execution, DSP, analysis, graph/runtime semantics, and
  runtime diagnostics.
- Pulse remains the authority for Loophole project/session state and editing.
- Finch remains the authority for app workflow, review UX, sidecar handling,
  and library-specific behaviour.
- Finch and Loophole consume Signal-owned crates rather than reimplementing
  core analysis logic locally.

## Invariants

- Reusable DSP and analysis do not live in Finch-local or Loophole-local
  wrapper code.
- Plugin and hardware integrations must not become the home of core DSP logic.
- Process boundaries follow trust and stability needs, not historical repo
  layout.
- Real-time paths avoid blocking, allocation churn, and unbounded work.
- Research authority for DSP and analysis topics lives in `docs/research/`.

## Time-Stretch Boundaries

- `OfflineHighQuality` is the frozen transparent production route; the
  successor program is closed under Contract `084`.
- `CreativeStretch` is a separate public offline whole-buffer API: neutral
  `Dream` at every exact target `4x..16x`, with `space` as its only creative
  control. `Cyclic` (exact `2x..8x`, cycle duration control) is a separate
  admitted character. No automatic routing or blend between them.
- `RealtimePreview` is a bounded, callback-safe prototype: proven and
  deliberately unadopted as a direct audio-thread source. See `g10.040`.
- The g10.036-g10.042 stretch audit lane is complete: Transparent defects
  fixed, cache identity `v3`, surface consolidated, resumable offline render
  admitted, pitch resumable render landed, and the chunked renderer plus its
  seam smoother deleted.

## Control Plane and Supervision

The control plane is `signal-runtime`: lifecycle, graph plan vocabulary,
plugin discovery/sandbox records, media pipeline, and observation reports. It
is not the audio callback — engine-block simulation, the prework scheduler,
and transport-session concurrency were removed in g10.020.

- Runtime-owned reports (`RuntimeControlSnapshot`, `RuntimeTimelineSnapshot`,
  `RuntimeEngineBlockSnapshot`, supervisor/observation surfaces) are the
  shared export vocabulary for hosts and automation.
- Recovery policy is "overlap then hand off": replacement broker sessions can
  be admitted before old transport teardown completes, and runtime owns the
  lingering-session cleanup state machines. The full rules live in
  `docs/contracts/002-supervisor-export-schema-and-report-boundary.md` and
  the g09/g10 logs — this page only records that runtime is the authority.

## Performance and Reliability Constraints

- Realtime-safe code paths must be deterministic and allocation-aware.
- Shared crates must remain usable in both runtime and offline-analysis
  contexts.
- Plugin code is untrusted by default; sandboxing is the preferred
  containment layer.
- Native shims are acceptable where ABI or platform constraints make them
  the lower-risk integration choice.

## Interfaces With Roadmaps

- `g11` is the active generation. `g10` stretch audit is complete through
  `g10.042`.
- CLAP, VST3, AU, and LV2 hosting is implemented through adapter crates,
  `signal-plugin-sandbox`, and `signal-plugin-bridge`. `g11.001` wired those
  backends through `signal-host-local`.
- SharedSandbox multiplexing closed in `g11.002`. Remaining plugin
  integration work is product-pulled workflow depth listed in
  `docs/roadmaps/backlog/post-g10-rebuild-on-demand.md`.

## Next Task

Stop for operator selection of the next Signal-only backlog pull. Do not start
a follow-on generation. `g11.001` and `g11.002` are complete. Linux CLAP
filesystem discovery (`086`) shipped 2026-08-21. Do not open `g12`.
