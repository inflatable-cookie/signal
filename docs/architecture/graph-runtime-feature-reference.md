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

Current host-I/O and hardware-portability tracking includes:

- backend-neutral hardware capability and negotiation primitives in
  `signal-hardware` for:
  - device descriptors
  - stream requests and negotiated stream configs
  - clock topology hints
  - lifecycle ownership and restart policy
  - clock source and latency profile
  - backend health and diagnostic counters
- runtime-owned hardware application through `HardwareConfigRequest`, with the
  active processing configuration and backend policy tracked in
  `EffectiveRuntimeConfig` and `RuntimeDiagnosticsSnapshot`
- host-augmented runtime receipts for:
  - negotiated hardware identity and stream contract
  - processing versus hardware sample-rate visibility
  - explicit `SameClock`, `CrossClock`, `Aggregate`, and `Degraded`
    clock-domain classification
  - explicit `Direct`, `RuntimeResampled`, `RecoveryConstrained`, and
    `Unconfigured` fallback-state classification
  - explicit host clock transition-state classification for first observation,
    aggregate entry, cross-clock entry, return-to-direct recovery, and other
    reconfiguration paths
  - clock source, lifecycle ownership, and restart policy
  - live latency and callback cadence
  - callback-pump state and output transfer counters
  - backend health, xrun, device-loss, and restart counters
- shared host observation/supervisor delivery through:
  - `RuntimeHostIoSummary`
  - `RuntimeHostObservationReport`
  - `RuntimeHostSupervisorReport`
- current same-clock live-path visibility through matching runtime versus
  negotiated host sample-rate reporting plus explicit host clock-domain export
- current cross-clock live-path visibility through explicit runtime-owned
  fallback-state export rather than backend-local inference
- current aggregate-clock live-path visibility through the same runtime-owned
  host clocking receipt family
- runtime-owned resampling on offline/export paths rather than backend-private
  sample-rate conversion
- current limitation: multi-member aggregate detail, drift compensation, and
  broader backend-matrix coverage are still not exposed through richer
  runtime-owned receipts, so those deeper backend details remain internal

Current plugin backend and delegation tracking includes:

- format-neutral plugin vocabulary in `signal-plugin` for:
  - descriptor, feature, bus, and parameter surfaces
  - state, processing, and lifecycle contracts
  - readiness and fault vocabulary
  - sandbox capability and transport abstractions
- runtime-owned plugin execution/export surfaces in `signal-runtime` for:
  - plugin scan/discovery receipts with typed root/filter intent
  - discovered-plugin catalog records carrying format-neutral identity,
    feature, I/O, state, processing, and lifecycle detail
  - plugin-backed node binding projection
  - plugin lifecycle, chain, recall, and compensation snapshots
  - typed plugin format/type identity carried through sandbox, recall, and
    delegated execution stage DTOs
  - delegated offline execution boundary, request, receipt, and merge/outcome
    families
- current adapter-specific CLAP realization in `signal-plugin-clap` for:
  - extension negotiation
  - discovered type and instance control surfaces
  - prepare and block protocol details
  - CLAP-specific event and shared-memory packet mapping
- current host-neutral rule: delegated execution fulfillment may happen in a
  host adapter, but ownership of stage identity, recall payload, completion
  status, and finalization receipts stays in runtime-owned DTOs
- current proof boundary: runtime public-boundary and supervisor-export
  fixtures now consume the widened discovery catalog without adapter-local
  reconstruction
- current limitation: broader backend-neutral capability projection and
  adapter coverage beyond the CLAP-first path are still explicitly deferred

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
- runtime-owned offline render queue and purge receipts for multi-request
  render advancement and artifact/report cleanup without rebuilding queue or
  purge meaning in host-local code
- runtime-owned deferred-service receipts for offline render queue execution,
  exposing typed `Run`, `Throttle`, and `Defer` decisions plus the runtime
  state that caused them
- runtime-owned deferred-service receipts for offline render purge plus
  observation/supervisor export of the latest deferred-service decision so
  consumers can inspect orchestration outcomes without private runtime state
- runtime-owned lingering transport cleanup queue visibility through
  `RuntimeTransportConcurrencySnapshot`, including pending cleanup waves and
  deferred retry work counts
- runtime-owned routed topology summaries that carry aggregated plugin-chain
  latency/compensation state per track lane, bus group, console group, and
  send/return route, plus node-level recall payload/status and compensation
  export that survives rebinding and clears cleanly on graph refresh
- runtime-owned anticipative prework forecasting, queueing, and service-state
  reporting
- runtime-owned widening of anticipative prework service budget from compatible
  schedule-stream capacity, with degraded/plugin/transport policy still able to
  throttle or yield that widened scope
- runtime-owned widening of requested anticipative service cadence from
  compatible schedule-stream capacity, with elevated pressure still collapsing
  widened requests back to the bounded safe scope
- runtime-owned schedule-projection refresh and forecast-plan churn rebuilds
  that reuse the widened service policy instead of dropping back to a separate
  single-cycle refresh path
- runtime-owned restart/reconfigure and mixed execution-class graph transitions
  that preserve the same schedule-width service policy and keep scheduler
  receipts coherent across lifecycle churn
- focused scheduler stress proofs for mixed graph churn, invalidation-heavy
  transition bursts, and constrained anticipative windows
- runtime-owned transport concurrency and lingering-cleanup workflow surfaces
- runtime-owned plugin discovery coverage receipts:
  - per-format discovery breadth through `RuntimePluginFormatCoverageRecord`
  - aggregate backend-neutral capability breadth through
    `RuntimePluginCapabilityCoverageSummary`
- contract-frozen scheduler inspection hierarchy:
  - `RuntimeEngineBlockSnapshot` as per-block execution truth
  - `RuntimeSchedulerSnapshot` as lifecycle/control-state truth
  - `RuntimeSchedulerExportSummary` as the narrow stable digest for reports and
    automation
  - `RuntimeExecutionTopologySummary` and `RuntimeSchedulerTopologySummary` as
    the explanatory topology context for those choices rather than a
    host-recomputed scheduler model
- contract-frozen shared host-edge tiers:
  - `LocalRuntimeHost::new`, `ServerRuntimeHost::new`, `RuntimeSupervisorApi`,
    and `supervisor_report()` as the first stable shared host edge
  - host-specific report enrichments, summary structs, scenario boot helpers,
    and local delegated executor helpers remain explicitly unstable until later
    `g05.002` tranches promote them
  - `signal-supervisor-tools --describe-host-edge-boundary` and
    `effigy acceptance:host-edge-consumer --repo .` as the machine-readable
    inspection and consumer-proof surface for that stable/unstable split

Planned elsewhere but not implemented in these crates yet:

- broader multichannel adaptation beyond mono/stereo
- richer graph stage catalogs and plugin-hosted stage execution inside
  `signal-graph` itself
- a production-grade multithreaded scheduler rather than the current embeddable
  block-processing shell
- true cost-aware or work-stealing dispatch balancing beyond schedule-stream
  width as a bounded proxy for multicore capacity
- long-duration scheduler threshold/fail-gate benchmark policy
- narrower, more task-specific runtime docs for each control subsystem

## Current Entry Points

Useful implementation entry points after this doc:

- `crates/signal-graph/src/lib.rs`
- `crates/signal-runtime/src/lib.rs`
- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-runtime/tests/public_contract_boundary.rs`
- `crates/signal-host-local/src/host.rs`
- `crates/signal-host-server/src/host.rs`
- `crates/signal-host-local/tests/public_host_edge_boundary.rs`
- `crates/signal-host-server/tests/public_host_edge_boundary.rs`
- `crates/signal-supervisor-tools/src/main.rs`
- `crates/signal-supervisor-tools/tests/public_packaging_manifest_boundary.rs`
- `crates/signal-runtime/examples/supervisor_report_demo.rs`
- `cargo run -p signal-supervisor-tools -- --describe-host-edge-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-downstream-automation --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-downstream-fail-gates --format=json`
- `effigy acceptance:host-edge-consumer --repo .`
- `effigy acceptance:plugin-backend-breadth --repo .`
- `effigy acceptance:conformance --repo .`
- `effigy acceptance:release-boundary --repo .`
- `effigy acceptance:packaging-manifest --repo .`
- `effigy acceptance:release-packaging-consumer --repo .`
- `effigy acceptance:downstream-release --repo .`
- `effigy acceptance:downstream-depth --repo .`
- `effigy acceptance:downstream-automation --repo .`
- `effigy acceptance:downstream-gate --repo .`
- `effigy acceptance:analysis --repo .`
- `cargo run -p signal-supervisor-tools -- --describe-generation-closeout --format=json`
- `effigy acceptance:g05-closeout --repo .`
- `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`
- `docs/contracts/010-publication-grade-packaging-manifest-and-release-receipt-contract.md`
- `docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md`

## Next Task

Continue `g06.001` with Batch 1.2 by applying the interruption and
resumability contract to active runtime-owned snapshots, receipts, and shared
host-edge surfaces before deeper recovery, profiling, plugin, hardware, and
media-service milestones widen the implementation.
