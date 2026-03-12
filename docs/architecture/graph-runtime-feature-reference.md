# Graph And Runtime Feature Reference

Status: active
Owner: core-product
Updated: 2026-03-12
Vision refs: `docs/vision/001-signal-vision.md`
Architecture refs: `docs/architecture/system-architecture.md`, `docs/architecture/package-map.md`

## Purpose

Document the graph-execution and runtime functionality that is implemented
today in the Rust workspace. This complements the DSP/analysis reference by
describing the current executable graph, scheduler-facing planning, runtime
control state, and supervisor-facing observation surfaces.

## Scope Summary

The implemented graph/runtime surface currently lives in these crates:

- `signal-graph`
- `signal-runtime`

This file is implementation-facing. It describes code paths and public result
surfaces that exist under `crates/` today, not roadmap intent.

## Graph Execution

### `signal-graph`

Primary surface:

- `GraphExecutionPlan`
- `ExecutableGraph`
- `GraphExecutionRequest`
- `GraphBlockReport`

Current stage model:

- stage specs:
  - `Gain`
  - `Bias`
  - `TanhDrive`
  - `StereoBalance`
  - `HardClip`
  - `LowPass`
  - `Delay`
- execution classes:
  - `PureTransform`
  - `Stateful`
  - `LatencyBearing`
  - `PluginBacked`

Current graph features:

- node-based execution plans with explicit node ids, stage stacks, latency, and
  tail metadata
- buffer contracts per node:
  - input and output bus ids
  - channel layouts
  - scratch-buffer counts
  - silence policy
  - channel adaptation mode
  - reset policy
- topology metadata per node:
  - role
  - track-lane id
  - bus-group id
  - console-group id
  - send/return id
- contract validation and contract summaries
- contract validation now treats missing topology ownership ids as explicit
  issues for track, bus, send/return, and console roles
- routed bus execution with:
  - direct edges
  - fan-in bus mixing
  - fan-out bus propagation
  - output latency and tail aggregation
- adaptive mono/stereo channel conversion for compatible contracts
- silence handling through process, bypass, or clear-output policies

### Planning And Dispatch

Current planning behavior:

- maps nodes into planning groups:
  - `InlineRealtime`
  - `StatefulRealtime`
  - `AnticipativeEligible`
- derives lane order from those groups:
  - `Anticipative`
  - `Realtime`
- emits dispatch summaries for phase and lane boundaries
- records planning summaries in block reports

Current execution behavior:

- processes blocks through an explicit `GraphExecutionContext`
- carries:
  - processing epoch
  - block sequence
  - projection epoch
  - parameter epoch
  - configured block size
  - anticipative-enabled flag
  - transport-playing flag
  - transport tempo
  - timeline position
- supports a prepared-dispatch concept through `GraphPreparedDispatch`
- reports whether work was prepared ahead of realtime and then handed off

### Parameter Event Application

Current graph parameter surface:

- `GraphParameterTarget`
- `GraphParameterEvent`
- `GraphParameterBatch`
- `GraphParameterApplicationStrategy::SplitAtEvents`

Current behavior:

- parameter events are block-local and sample-offset based
- targets identify a node, stage index, and stage parameter
- event application currently supports split-at-events processing with a capped
  sub-block count
- unsupported or mismatched events are counted rather than crashing execution
- block reports include:
  - parameter event count
  - targeted node count
  - ignored event count
  - sub-block count
  - coalesced event count

Current stage parameters:

- gain linear
- bias amount
- tanh drive
- stereo balance
- hard-clip threshold
- low-pass cutoff
- delay feedback

### Graph Output Surface

`GraphBlockReport` currently exposes:

- node/execution-class counts
- contract/routing summary counts
- planning phase and lane order
- dispatch counts and order
- dynamic-kernel stage counts
- latency and tail maxima
- parameter application summary
- frame and channel counts
- input, prepared-dispatch, realtime-input, and output peak metrics
- output RMS and first output sample

Current constraints:

- block execution is still offline/in-memory rather than a dedicated realtime
  scheduler thread model
- channel adaptation is limited to mono/stereo cases
- stage catalog is intentionally small and host-independent
- plugin-backed nodes are represented in planning and reporting, but graph is
  not itself a plugin host

## Runtime Orchestration

### `signal-runtime`

Primary surface:

- `SignalRuntime`
- `RuntimeConfig`
- runtime-host interfaces under `interfaces.rs`

Current runtime features:

- embeddable runtime profiles:
  - `Local`
  - `Server`
- handshake, configure, start, stop, restart, and safe-mode control
- graph projection receipt and projection epoch tracking
- schedule, transport, parameter-batch, plugin-binding, and automation
  projection application
- runtime-owned execution of `signal-graph` blocks
- runtime-owned engine, control, timeline, automation, diagnostics, and
  supervision snapshots

### Current Runtime Control And Observation Surfaces

Key observation structs include:

- `RuntimeControlSnapshot`
- `RuntimeTimelineSnapshot`
- `RuntimeAutomationSnapshot`
- `RuntimeEngineBlockSnapshot`
- `RuntimeMeteringSnapshot`
- `RuntimeExecutionTopologySummary`
- `RuntimeTransportConcurrencySnapshot`
- `RuntimeSupervisionSnapshot`
- `RuntimeDiagnosticsSnapshot`

Current control-state tracking includes:

- handshake/configure/start/stop/restart counts
- last client version
- last stop reason
- last reconfigure request
- current readiness state
- effective runtime config

Current timeline tracking includes:

- next block sequence
- block-sequence continuity
- transport epoch
- last transport transition kind
- transport start/stop/seek/tempo/loop transitions
- last engine block start/end sample positions
- loop-wrap count

Current automation tracking includes:

- projected automation lane, point, and segment counts
- typed automation targets as explicit `node_id` plus `parameter_id` pairs
- explicit interpolation families:
  - `Hold`
  - `Linear`
- per-lane playback policy:
  - `ramp_step_samples`
  - `max_sub_blocks`
- mapped versus unmapped projected-lane counts
- hold-lane versus linear-lane counts
- last projected batch strategy, ramp-step, and sample-offset summaries
- parameter event and modulation counts
- gesture counts
- first/last values
- first/last epochs
- segment counts and lease rollovers

Current tempo-map and warp tracking includes:

- runtime-owned tempo-map segments with explicit `Hold` and `Linear` timing
  interpolation
- resolved project-tempo source tracking:
  - `DefaultFallback`
  - `TransportProjection`
  - `TempoMapSegment`
- active tempo-map segment identity, next-segment visibility, and resolved
  project tempo at the current transport position
- warp clip readiness and realized ratio summaries that preserve the source
  tempo, resolved project tempo, and active tempo-map segment provenance
- warp degraded-state visibility for unsupported ratios, missing source tempo,
  missing media assets, and not-ready cached media
- shared tempo-map and warp export carried through:
  - `RuntimeObservationReport`
  - `RuntimeSupervisorReport`
  - host observation/supervisor JSON surfaces

Current clip-processing tracking includes:

- runtime-owned clip fade envelopes with explicit shape families:
  - `Linear`
  - `EqualPower`
  - `SmoothStep`
- runtime-owned clip gain envelopes with explicit shape families:
  - `Hold`
  - `Linear`
- ordered clip-treatment stages carried per clip:
  - `Warp`
  - `FadeIn`
  - `GainShape`
  - `FadeOut`
- clip-processing readiness and validation for:
  - missing media assets
  - not-yet-realized warp state
  - degraded warp state
  - invalid gain-envelope or fade-duration requests
- clip-processing snapshots that preserve realized warp ratio plus project-tempo
  provenance alongside the fade/gain treatment order
- runtime-owned clip-render request/result seam that applies fade and gain
  envelopes against timeline-relative clip positions on provided post-warp
  buffers
- render-path validation that:
  - silences samples outside the clip bounds
  - enforces post-warp input for warp-enabled clip renders
  - preserves the same treatment-order metadata used by observation/export
- shared clip-processing export carried through:
  - `RuntimeObservationReport`
  - `RuntimeSupervisorReport`
  - host observation/supervisor JSON surfaces

Current offline-render contract tracking includes:

- runtime-owned offline render request surfaces:
  - `RuntimeOfflineRenderRequest`
  - `RuntimeOfflineRenderStemTarget`
  - `RuntimeOfflineFreezeArtifactRequest`
- runtime-owned preview surfaces:
  - `RuntimeOfflineRenderContractPreview`
  - `RuntimeOfflineRenderStemPreview`
  - `RuntimeOfflineFreezeArtifactPreview`
- runtime-owned render result surfaces:
  - `RuntimeOfflineRenderResult`
  - `RuntimeOfflineRenderStemResult`
  - `RuntimeOfflineFreezeArtifactResult`
- target resolution for:
  - main mix
  - track lanes
  - bus groups
  - console groups
  - send/return groups
- contract-preview derivation that reuses runtime-owned:
  - routed execution topology
  - clip-processing pipeline snapshots
  - tempo-map snapshots
  - plugin recall handoff selections
- first offline-render engine path that:
  - decodes runtime-cached WAV media assets
  - reapplies clip fade/gain treatment through the existing clip-render seam
  - executes the graph to produce main mix output
  - captures requested routed bus outputs for stems
  - clones rendered stem audio into freeze artifacts while preserving recall
    handoff metadata
- freeze artifact preview that keeps recall ownership in
  `RuntimePluginRecallHandoffSnapshot` and resolves stable handoff stage ids
  rather than requiring supervisor/export parsing
- current proof-path constraints:
  - export sample rate must match the runtime sample rate
  - media decode is currently WAV-only
  - plugin-backed offline stages reuse cached render overrides rather than
    driving a dedicated offline sandbox pass

Current metering and diagnostics export includes:

- flat meter-source snapshots with explicit track-lane, bus-group,
  console-group, and send/return ownership ids
- loudness-oriented runtime export for:
  - main output peak/RMS
  - momentary loudness
  - short-term loudness
  - integrated loudness
  - clipped-sample counts
- routed meter aggregation on top of `RuntimeExecutionTopologySummary` for:
  - track lanes
  - bus groups
  - console groups
  - send/return groups
- shared metering export carried through:
  - `RuntimeObservationReport`
  - `RuntimeSupervisorReport`
  - host observation/supervisor JSON surfaces

## Runtime Scheduler And Prework

Current engine snapshot features:

- graph/planning summary mirrored from `signal-graph`
- scheduler topology summary for track lanes, bus groups, send/return groups,
  and console groups
- routed mixer execution summary with:
  - lane summaries
  - node summaries
  - track-lane ownership summaries
  - bus-group summaries
  - send/return summaries
  - console-group summaries
- anticipative planning and dispatch counts
- prepared-dispatch versus realtime-dispatch metrics
- prework cache state, queue depth, and queue capacity
- pending-target backlog by class:
  - `Immediate`
  - `NearTerm`
  - `Deferred`
- prework service state:
  - `Disabled`
  - `Idle`
  - `Pending`
  - `Servicing`
  - `Yielding`
  - `Paused`
  - `Starved`
- prework service pressure:
  - `Normal`
  - `Elevated`
  - `Critical`
- semantic service policy:
  - `Balanced`
  - `LatencyFocused`
  - `PluginConstrained`
- plugin-gating and transport-gating counters
- service-cycle, throttle, yield, pause, resume, and starvation counters
- forecast mode, profile, and policy visibility

Current runtime scheduling behavior:

- runtime derives future execution targets from runtime-owned forecast state
- pending future targets are serviced separately from current realtime block
  application
- compatible queued prework can be preserved while incompatible entries are
  invalidated or retired
- prework freshness is tracked in block-sequence space
- transport transitions and parameter-batch application can invalidate queued
  prework

## Runtime Transport Concurrency

Current transport-concurrency surface includes:

- active session tracking for steady-state and recovery-overlap attaches
- lingering cleanup planning and queue receipts
- pending cleanup-wave summaries
- cleanup attempt counters and cleanup error visibility
- peak attached, overlap, and lingering session counts

Current behavior:

- runtime remains authoritative for session admission state
- lingering cleanup work is runtime-queued and processing-epoch aware
- cleanup plans are emitted as typed runtime work rather than raw host-local
  snapshot interpretation

## What Is Implemented Versus Planned

Implemented now:

- routed executable graph plans with explicit node contracts and bus routing
- planning groups, execution lanes, and dispatch-order reporting
- block-local sample-offset parameter-event application
- runtime-owned graph execution and engine-block observation
- runtime-owned routed metering, loudness snapshots, and supervisor-facing
  export on explicit mixer topology
- runtime-owned typed automation playback projections with hold/linear
  interpolation, per-lane resolution policy, and multi-block parameter-batch
  realization through `signal-graph`
- runtime-owned tempo-map projections and warp clip snapshots with explicit
  timing intent versus realized project-tempo source, plus degraded/fallback
  reporting on top of media-cache readiness
- runtime-owned clip-render request/result seam that applies fade and gain
  envelopes through typed clip-treatment contracts instead of host-local media
  logic
- runtime-owned plugin-chain snapshots that preserve planned chain order,
  realized per-stage latency/tail state, compensation readiness, and typed
  recall payload/status export through observation and supervisor reports
- runtime-owned plugin recall handoff snapshots that separate authoritative
  recall payload from export-only summary fields for later offline
  render/freeze consumers
- runtime-owned offline render request and contract-preview surfaces that let
  later render/freeze callers resolve stem topology, clip readiness, tempo, and
  plugin recall dependencies directly from runtime-owned state before a full
  engine path exists
- runtime-owned offline render result surfaces plus a first block-based engine
  path that combines runtime media cache access, clip treatment, graph
  execution, captured bus output, and recall-backed freeze export without
  relying on supervisor/export parsing
- runtime-owned offline render artifact/report receipt surfaces that can
  materialize exported main-mix, stem, and freeze artifacts under a request-
  owned artifact root while preserving runtime-frame versus exported-frame
  accounting and export sample-rate conversion
- runtime-owned offline media decode that can read broader cached asset
  formats during render preparation, plus fresh-only live plugin override use
  that falls back to the Signal-owned plugin stage model when cached plugin
  output is stale
- runtime-owned offline render manifest bundles plus explicit plugin execution
  boundary export that let downstream consumers package artifacts and inspect
  which stages remain Signal-modeled versus which ones require later
  host-delegated offline execution
- runtime-owned delegated offline plugin execution request/result receipts that
  derive from the execution boundary and fold back into the same manifest
  bundle instead of escaping into a host-local packaging model
- runtime-owned delegated execution materialization that rewrites the same
  report/manifest delivery bundle after delegated receipts land, keeping
  runtime export aligned with the in-memory handoff state
- runtime-owned delegated executor outcome/merge contracts that let later host
  adapters feed replacement main-mix, stem, and freeze outputs back into
  runtime-owned finalization instead of creating a parallel export path
- `signal-host-local` delegated executor adapter wiring that prepares one
  concrete host-side delegated outcome and round-trips it through the same
  runtime-owned offline manifest/report finalization path without rebuilding
  export surfaces in host code
- runtime-owned profiling and soak receipts derived from supervisor and
  host-observation reports so routed engine harnesses can compare timing,
  callback, xrun, restart, and recovery counters without inventing host-local
  benchmarking schemas
- runtime-owned degradation-expanded live profiling/soak receipts plus typed
  offline-render profiling/soak receipts that pin routing gates, plugin-chain
  quarantine/unavailability, and delegated-offline degraded outcomes through
  the same reusable runtime-owned hardening contract
- runtime-owned routed topology summaries that carry aggregated plugin-chain
  latency/compensation state per track lane, bus group, console group, and
  send/return route, plus node-level recall payload/status and compensation
  export that survives rebinding and clears cleanly on graph refresh
- runtime-owned anticipative prework forecasting, queueing, and service-state
  reporting
- runtime-owned transport concurrency and lingering-cleanup workflow surfaces

Planned elsewhere but not implemented in these crates yet:

- broader multichannel adaptation beyond mono/stereo
- richer graph stage catalogs and plugin-hosted stage execution inside
  `signal-graph` itself
- a production-grade multithreaded scheduler rather than the current embeddable
  block-processing shell
- narrower, more task-specific runtime docs for each control subsystem

## Current Entry Points

Useful implementation entry points after this doc:

- `crates/signal-graph/src/lib.rs`
- `crates/signal-runtime/src/lib.rs`
- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-host-local/src/host.rs`
- `crates/signal-supervisor-tools/src/main.rs`
- `crates/signal-runtime/examples/supervisor_report_demo.rs`

## Next Task

COMPLETE. `g03` closed after the runtime-owned hardening receipts were proven
across routing, plugin-chain, and offline-render fault paths.
