# Graph And Runtime Feature Reference

Status: active
Owner: core-product
Updated: 2026-03-16
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
- contract-frozen device supervision rule:
  - runtime supervision remains authoritative for recovering versus faulted
    hardware state and later recovery exhaustion meaning
  - backend diagnostics and callback loss are additive evidence rather than a
    competing consumer-facing restart taxonomy
  - later clock-domain, endpoint-topology, and external-I/O work must build on
    the same supervision substrate instead of redefining hardware fault
    ownership
- contract-frozen drift and endpoint-topology rule:
  - runtime-owned receipts remain authoritative for consumer-visible drift,
    discontinuity, duplex mismatch, partial availability, and endpoint-topology
    meaning
  - backend timestamps, callback deltas, and device lists remain contributing
    evidence rather than a competing topology or drift model
  - later external-I/O, monitoring, and loopback work must deepen this shared
    topology contract instead of inventing host-local endpoint semantics
- contract-frozen monitoring and loopback rule:
  - runtime-owned receipts will remain authoritative for external-I/O role,
    monitor tap-point, loopback, and bounded measurement meaning
  - hardware callbacks, endpoint inventories, and product-local routing labels
    remain contributing evidence rather than a competing monitor-path model
  - later calibration, waveform, and media-service work must deepen the shared
    monitoring contract instead of moving loopback truth into host-local code
- current runtime-owned device supervision receipt depth through:
  - `RuntimeDeviceSupervisionSnapshot` on runtime observation and supervisor
    report surfaces
  - explicit supervision state, restart state, and hardware fault-boundary
    classification
  - additive host-fed evidence for device-loss counts, restart attempts,
    restart failures, watchdog restarts, restart policy, backend health, stream
    state, and active device identity
  - machine-readable boundary proof through
    `signal.runtime.device-supervision-boundary` and
    `effigy acceptance:device-supervision-boundary`
- current aggregate-clock live-path visibility through the same runtime-owned
  host clocking receipt family
- current drift and endpoint-topology receipt depth through:
  - `RuntimeHostClockingSummary` carrying explicit:
    - `drift_state`
    - `discontinuity_state`
    - `duplex_mismatch_state`
    - `endpoint_topology`
    - `partial_availability`
  - `RuntimeExternalIoSnapshot` preserving the same bounded drift or topology
    meaning instead of collapsing to fallback-only health
  - `signal-host-local` deriving the shared fields from the active stream
    contract, backend health, transition state, and stream state in one place
  - shared host-edge alignment that now reuses one `host_io` receipt per
    outward report instead of recomputing divergent first-observation
    transition states
  - machine-readable boundary proof through
    `signal.runtime.clock-topology-boundary` and
    `effigy acceptance:clock-topology-boundary`
- current external-I/O monitoring and loopback receipt depth through:
  - `RuntimeExternalIoSnapshot` carrying explicit:
    - `health_state`
    - `device_change_state`
    - `primary_role`
    - `monitoring_state`
    - `monitoring_tap_point`
    - `loopback_state`
  - `RuntimeObservationReport` exporting the same bounded snapshot even when
    live host-I/O context is unavailable
  - `signal-host-local` mapping live host-I/O state into the shared runtime
    receipt family instead of a host-private monitor model
  - `signal-host-server` exporting the same receipt shape with explicit
    `Unavailable` classifications where live monitoring state is not present
  - machine-readable boundary proof through
    `signal.runtime.external-io-boundary` and
    `effigy acceptance:external-io-boundary`
- contract-frozen media-service rule:
  - runtime-owned media asset identity, indexing, invalidation, waveform
    readiness, preview readiness, and analysis-ready state remain authoritative
    for reusable consumers
  - shared `signal-analysis*` crates own algorithm families and result types,
    while `signal-runtime` owns the service-state boundary
  - the bounded reusable media-service seam is now frozen in
    `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`
  - later library, waveform, preview, and metadata work must deepen that
    shared contract instead of moving preview or cache readiness back into
    product-local code
- implemented media-service baseline depth:
  - `RuntimeObservationReport` now carries runtime-owned
    `media_pipeline_snapshot` and `media_service_snapshot`
  - `RuntimeSupervisorReport` and the shared local/server `supervisor_report()`
    paths now expose the same indexing, invalidation, waveform, and preview
    readiness state
  - the media pipeline is no longer only a direct runtime API seam; it now
    participates in shared observation and export surfaces
  - machine-readable boundary proof now exists through
    `signal.runtime.media-service-boundary` and
    `effigy acceptance:media-service-boundary`
- contract-frozen analysis-metadata rule:
  - reusable asset-analysis descriptors and library-service meaning must stay
    aligned with runtime-owned media indexing, waveform, preview, and
    invalidation receipts rather than product-local metadata tables
  - shared `signal-analysis*` crates remain the algorithm family authority,
    while `signal-runtime` owns reusable descriptor readiness, staleness, and
    bounded family-coverage meaning
  - the bounded reusable analysis-metadata seam is now frozen in
    `docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md`
  - later metadata extraction and library-service depth must widen that shared
    contract instead of rebuilding private product extraction pipelines
- implemented reusable metadata baseline:
  - `RuntimeObservationReport` and `RuntimeSupervisorReport` now carry
    `media_library_snapshot` alongside the earlier media pipeline and service
    snapshots
  - `RuntimeMediaLibraryServiceSnapshot` now exposes per-asset
    `RuntimeMediaLibraryAssetDescriptor` records with runtime-owned descriptor
    state plus the first real bounded payload depth:
    `RuntimeMediaLoudnessDescriptor` and `RuntimeMediaCharacterDescriptor`
  - loudness and character family coverage can be `Ready`, while rhythm,
    tonal, and embedding stay explicitly `Deferred` instead of being inferred
    from product-local extraction gaps
  - shared local and server host reports now expose the same library-service
    descriptor family, including explicit `Unavailable` outcomes when indexed
    media is not analyzable
  - machine-readable consumer-boundary proof now exists through
    `signal.runtime.analysis-metadata-boundary` and
    `effigy acceptance:analysis-metadata-boundary`
- contract-frozen integrated acceptance lane policy:
  - the first bounded shared fault-injection and multi-backend acceptance
    contract now composes the already-closed recovery, timing, adapter,
    hardware, media-service, and analysis-metadata boundaries
  - integrated acceptance depth is now explicitly split into `required`,
    `advisory`, and `deferred` tiers so later harness work does not hide
    unstable or long-session soak paths inside the bounded lane
  - later `g06.019` implementation must build a machine-readable integrated
    harness descriptor and Effigy lane on top of that policy before `g06.020`
    widens into promotion and soak gates
  - the bounded policy is now frozen in
    `docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md`
- implemented integrated acceptance lane baseline:
  - `signal-supervisor-tools` now exposes the machine-readable
    `signal.runtime.integrated-acceptance-lane` descriptor with explicit
    `required`, `advisory`, and deferred depth
  - Effigy now owns `acceptance:integrated-acceptance-lane`, grouping the
    required cross-family path across interruption, diagnostics, scheduling,
    plugin continuity, parity, supervision, external-I/O, media-service, and
    analysis-metadata boundaries
  - the grouped lane also repaired stale watchdog-restart expectations in the
    interruption proofs so the required path stays aligned with the current
    runtime safe-mode restart threshold
- integrated acceptance lane now has cross-family export proof:
  - `signal-supervisor-tools` now proves one `signal.supervisor.export`
    artifact can carry recovery, deferred-work, adapter breadth, hardware,
    and media/library receipts together
  - the integrated acceptance descriptor and Effigy lane now both point at
    that shared export proof explicitly instead of only enumerating the
    milestone-local boundary tasks they compose
- contract-frozen `g06` closeout and soak policy:
  - the final `g06` closeout authority is now frozen as bounded soak plus
    promotion-gate policy layered on top of the integrated acceptance lane
  - required, advisory, and deferred closeout evidence is now explicit, with
    the integrated lane fixed as the required fast-path base and broader
    rerun, remote, and unstable overlap-heavy depth kept outside the required
    gate
  - the bounded policy is now frozen in
    `docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md`
- implemented `g06` soak lane and closeout gate baseline:
  - `signal-supervisor-tools` now exposes the machine-readable
    `signal.g06.long-session-soak-lane` descriptor alongside an updated `g06`
    generation-closeout descriptor
  - Effigy now owns `acceptance:g06-soak-lane` and `acceptance:g06-closeout`,
    making the bounded closeout gate runnable instead of policy-only
  - the soak lane keeps local `soak` and `mixed` required, keeps the
    integrated acceptance lane visible as advisory context, and leaves the
    broader `server soak` path explicitly deferred because the recovery-overlap
    attach-limit issue is still unstable
  - the generation-closeout descriptor now reports `g06`-specific residual
    risks and pending-readiness-review status instead of carrying forward the
    stale earlier-generation release shape
- `g06` closeout now carries an explicit promotion verdict:
  - the generation-closeout descriptor now resolves to `promote-g07` with all
    Loophole-facing readiness areas at `sufficient-for-promotion`
  - this closes `g06` as a reusable hardening and baseline-breadth generation
    without turning the closeout into a product-launch claim
  - unstable broader `server soak` and wider advisory rerun depth stay visible
    as deferred scope while `g07` becomes the single active queue
    unstable or product-local depth kept out of the final gate
  - the closeout policy is now frozen in
    `docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md`
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
- current adapter-specific VST3 realization in `signal-plugin-vst3` for:
  - platform scan-root presets across macOS, Linux, and Windows
  - VST3 class/controller pairing and descriptor projection
  - bounded shared-memory session planning against the shared runtime contract
- current host-neutral rule: delegated execution fulfillment may happen in a
  host adapter, but ownership of stage identity, recall payload, completion
  status, and finalization receipts stays in runtime-owned DTOs
- contract-frozen VST3 alignment rule:
  - `PluginFormat::Vst3` stays only the shared backend identity tag
  - VST3 module/class, component/controller, and Linux scan/load detail now
    widen shared Signal-owned discovery and lifecycle receipts through the new
    adapter baseline rather than a parallel adapter-local lifecycle taxonomy
- current VST3 host/runtime baseline:
  - local and server hosts now feed VST3 discovery into
    `RuntimePluginDiscoverySnapshot`
  - VST3 sandbox ensure now records shared runtime lifecycle, instance-state,
    and transport receipts on both hosts
  - Linux-hosted VST3 roots are now explicit through the server-host proof path
- current proof boundary: runtime public-boundary, stable host-edge, and
  supervisor descriptor fixtures now consume VST3 discovery and lifecycle truth
  without adapter-local reconstruction through the
  `signal.runtime.vst3-boundary` acceptance seam
- next adapter breadth contract:
  - AU discovery, lifecycle, macOS-scoped support claims, and future
    `signal-plugin-au` realization are now frozen against the same
    backend-neutral runtime-owned discovery and lifecycle seams rather than a
    product-local AU wrapper model
- current adapter-specific AU realization in `signal-plugin-au` for:
  - macOS component-root presets
  - Audio Unit component identity projection
  - bounded shared-memory session planning against the shared runtime contract
- current AU host/runtime baseline:
  - local and server hosts now feed AU discovery into
    `RuntimePluginDiscoverySnapshot`
  - AU sandbox ensure now records shared runtime lifecycle, instance-state, and
    transport receipts on both hosts
  - macOS-scoped AU roots are now explicit through focused host proof paths
- current AU proof boundary:
  - runtime public-boundary, stable host-edge, and supervisor descriptor
    fixtures now consume AU discovery and lifecycle truth without
    adapter-local reconstruction through the `signal.runtime.au-boundary`
    acceptance seam
- current limitation: broader backend-neutral capability projection and
  conformance beyond the new CLAP+VST3+AU baseline are still explicitly
  deferred
- contract-frozen cross-adapter parity rule:
  - CLAP, VST3, and AU now have one bounded parity vocabulary separating
    portable, format-guarded, adapter-private, and unsupported scope
  - Linux plugin breadth is now explicitly guarded rather than implied by
    adapter existence
  - later runtime parity receipts must stay inside the existing discovery,
    lifecycle, and supervisor/export families rather than a host-local
    portability matrix
- current runtime parity receipt family:
  - `RuntimePluginScanReceipt`, `RuntimePluginDiscoverySnapshot`, and
    `RuntimePluginLifecycleSnapshot` now carry per-format parity coverage with
    platform scope, placement-rule counts, active-transport state, and
    degraded or faulted lifecycle counts
  - hosts now seed the same CLAP, VST3, and AU platform coverage into runtime
    so Linux breadth and AU macOS-only scope stay explicit on shared receipts
    instead of host-private matrices
- current cross-adapter parity proof boundary:
  - public runtime, stable host-edge, and supervisor descriptor fixtures now
    consume the bounded parity receipt family without host-local portability
    matrices through the `signal.runtime.cross-adapter-parity-boundary`
    acceptance seam
- contract-frozen generic event rule:
  - `signal-plugin` now explicitly owns the bounded shared event vocabulary
    through `PluginEvent` and `EventPacket`
  - parameter value, parameter modulation, parameter gesture, note,
    note-expression, and three-byte MIDI events are now the first shared
    portable event families
  - `PluginProcessingContract` now carries explicit
    `supports_note_expression` capability across CLAP, VST3, AU, and runtime
    discovery receipts instead of inferring note-expression depth indirectly
  - `RuntimePluginEventSnapshot` now carries bounded last-batch and aggregate
    generic event continuity for parameter, note, note-expression, and MIDI
    output on runtime observation and supervisor surfaces
  - local and server host processing now feed `EventPacket::summary()` back
    into runtime-owned generic event receipts instead of keeping that widened
    event truth only in host-private payload summaries
  - the widened generic event, note-expression, and capability receipts are
    now proven consumable through public runtime, stable host-edge, and
    machine-readable supervisor surfaces via the
    `signal.runtime.generic-event-boundary` acceptance seam
  - richer packet families, editor semantics, controller mapping, and deeper
    per-format event models remain deferred instead of being implied by the
    CLAP-first path

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
- contract-frozen preset/interchange/ARA rule:
  - runtime recall payload remains the authority for later portable recall
    classification rather than adapter-native preset blobs or host storage
    location
  - preset descriptors are explicitly descriptive and non-authoritative until
    later runtime-owned interchange payload receipts exist
  - ARA-capable work is now bounded to document, source, and region context
    descriptors instead of product-local editor or arrangement semantics
  - later runtime/export depth must classify outcomes as `Portable`,
    `Guarded`, `NativeOnly`, `ContextOnly`, or `Unsupported` rather than
    inventing portability heuristics in host code
- runtime-owned preset/interchange/ARA receipt depth:
  - `RuntimePluginRecallPayload` now carries typed interchange classification,
    optional preset descriptor, and optional bounded ARA document/source/region
    context on the same recall path already used by plugin-chain snapshots,
    execution topology summaries, and offline render boundaries
  - stable host-edge `supervisor_report()` delivery now forwards the widened
    recall payload without adapter-local preset or ARA taxonomy
- runtime-owned recall portability consumer boundary:
  - downstream-style runtime proofs, both stable host edges, and
    `signal-supervisor-tools` now expose the
    `signal.runtime.recall-portability-boundary` acceptance seam for portable
    versus non-portable recall outcomes and bounded ARA-context transfer
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
- runtime-owned offline render session snapshots that preserve active and last
  render continuity, checkpoints, cancellation, and purge receipts through
  observation and supervisor export instead of filesystem or queue-ledger
  inference
- repo-owned `signal.runtime.offline-render-continuity-boundary` descriptor
  plus acceptance task so consumers can inspect resumable, restartable, and
  terminal render-session proof surfaces without private runtime or host code
- runtime-owned deferred-service receipts for offline render queue execution,
  exposing typed `Run`, `Throttle`, and `Defer` decisions plus the runtime
  state that caused them
- runtime-owned deferred-service receipts for offline render purge plus
  observation/supervisor export of the latest deferred-service decision so
  consumers can inspect orchestration outcomes without private runtime state
- runtime-owned lingering transport cleanup queue visibility through
  `RuntimeTransportConcurrencySnapshot`, including pending cleanup waves and
  deferred retry work counts
- contract-frozen deferred-work scheduler-policy hierarchy:
  - runtime-owned deferred-work classes and priority bands
  - starvation, backpressure, and cancellation as Signal-owned scheduler terms
  - per-block timing and bounded hotspot receipts as explicit policy context
  - host or product queue state remaining advisory rather than authoritative
- runtime-owned deferred-service receipts that now carry typed priority-band,
  blocking-priority, backpressure-source, starvation, and cancellation fields,
  with the same policy state preserved in `RuntimePerformanceSnapshot` and
  `RuntimePerformanceTraceReceipt`
- public runtime, stable host-edge, and `signal-supervisor-tools` boundary
  proof for the widened deferred-work scheduler-policy seam through
  `signal.runtime.deferred-work-policy-boundary`
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
- contract-frozen timing and pressure measurement hierarchy:
  - `RuntimeEngineBlockSnapshot` as the authoritative per-block timing and
    budget observation seam
  - `RuntimeSchedulerSnapshot` as the control-state companion that explains the
    measured block without replacing it
  - `RuntimePerformanceSnapshot` and `RuntimePerformanceTraceReceipt` as the
    bounded consumer and automation digests for timing and pressure
  - host callback cadence and backend timing remain additive evidence rather
    than a competing timing authority
- implemented bounded per-block timing instrumentation:
  - runtime-owned `RuntimeBlockDeadlinePressure`
  - latest block execution duration, derived deadline budget, utilization, and
    overrun amount on `RuntimeEngineBlockSnapshot`
  - aligned timing digest on `RuntimeBlockExecutionSummary`,
    `RuntimePerformanceSnapshot`, and `RuntimePerformanceTraceReceipt`
  - measured block execution now feeds runtime `cpu_load_percent` and
    `graph_latency_ms` once actual blocks have been processed
  - consumer-facing boundary proof via public runtime reports, stable
    host-edge `supervisor_report()`, and
    `signal-supervisor-tools --describe-block-timing-boundary`
- contract-frozen bounded hotspot and worker-lane instrumentation hierarchy:
  - `RuntimeEngineBlockSnapshot` as the explanatory graph, planning-group,
    lane-order, and dispatch-shape authority
  - `RuntimePerformanceSnapshot` as the bounded consumer digest for
    `hot_latency_node_*`, `hot_latency_group*`, and scheduler lane or handoff
    width context
  - `RuntimePerformanceTraceReceipt` as the bounded peak-hotspot digest across
    an observation window
  - host callback cadence and OS scheduler detail remain advisory evidence
    rather than a competing hotspot authority
- implemented bounded graph hotspot and worker-lane instrumentation depth:
  - `RuntimePerformanceSnapshot` now exposes:
    - hot-group membership count through `hot_latency_group_node_count`
    - critical-path lane identity and lane-level node, plugin, planning-group,
      and total-latency counts
    - typed `worker_lane_summaries` through
      `RuntimeWorkerLaneInstrumentationSummary`
  - `RuntimePerformanceTraceReceipt` now preserves:
    - peak hot-group membership count
    - peak critical-path lane identity
    - peak critical-path lane node, plugin, and total-latency counts
  - the widened hotspot and lane receipts are still derived from
    `RuntimeEngineBlockSnapshot` planning and lane-order truth rather than
    host-side scheduler or thread reinterpretation
  - consumer-facing boundary proof via:
    - public runtime consumption of `RuntimePerformanceSnapshot` and
      `RuntimePerformanceTraceReceipt`
    - stable local and server host-edge `supervisor_report().performance_snapshot()`
    - `signal-supervisor-tools --describe-critical-path-boundary`
    - `effigy acceptance:critical-path-boundary`
- contract-frozen shared host-edge tiers:
  - `LocalRuntimeHost::new`, `ServerRuntimeHost::new`, `RuntimeSupervisorApi`,
    and `supervisor_report()` as the first stable shared host edge
  - host-specific report enrichments, summary structs, scenario boot helpers,
    and local delegated executor helpers remain explicitly unstable until later
    `g05.002` tranches promote them
  - `signal-supervisor-tools --describe-host-edge-boundary` and
    `effigy acceptance:host-edge-consumer` as the machine-readable
    inspection and consumer-proof surface for that stable/unstable split

Planned elsewhere but not implemented in these crates yet:

- `g07.001` now freezes the canonical multichannel layout and channel-role
  contract on top of the current narrow `ChannelLayout` primitive, so broader
  multichannel meaning is no longer purely implicit even though the runtime and
  graph surfaces are still mostly mono/stereo in practice
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
- `cargo run -p signal-supervisor-tools -- --describe-block-timing-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-critical-path-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-downstream-automation --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-downstream-fail-gates --format=json`
- `effigy acceptance:host-edge-consumer`
- `effigy acceptance:block-timing-boundary`
- `effigy acceptance:critical-path-boundary`
- `effigy acceptance:plugin-continuity`
- `effigy acceptance:plugin-backend-breadth`
- `effigy acceptance:cross-adapter-parity-boundary`
- `effigy acceptance:conformance`
- `effigy acceptance:release-boundary`
- `effigy acceptance:packaging-manifest`
- `effigy acceptance:release-packaging-consumer`
- `effigy acceptance:downstream-release`
- `effigy acceptance:downstream-depth`
- `effigy acceptance:downstream-automation`
- `effigy acceptance:downstream-gate`
- `effigy acceptance:analysis`
- `cargo run -p signal-supervisor-tools -- --describe-generation-closeout --format=json`
- `effigy acceptance:g06-closeout`
- `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`
- `docs/contracts/010-publication-grade-packaging-manifest-and-release-receipt-contract.md`
- `docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md`
- `docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md`
- `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`
- `docs/contracts/015-offline-render-recovery-and-resumability-contract.md`
- `docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`
- `docs/contracts/018-graph-critical-path-hot-node-and-worker-lane-instrumentation-contract.md`
- `docs/contracts/019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md`
- `docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md`
- `docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md`
- `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`
- `docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md`
- `crates/signal-plugin-vst3/src/lib.rs`
- `crates/signal-supervisor-tools/src/main.rs`
- `effigy.toml`
- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-runtime/src/lib.rs`
- `crates/signal-runtime/tests/public_contract_boundary.rs`
- `crates/signal-host-local/tests/public_host_edge_boundary.rs`
- `crates/signal-host-server/tests/public_host_edge_boundary.rs`

## Next Task

Continue `g07.001` with Batch 1.2 by threading the canonical multichannel
layout and channel-role meaning through runtime-owned topology, hardware, and
plugin-facing receipts before the public proof batch closes the milestone.
