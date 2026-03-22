# Graph And Runtime Feature Reference

Status: active
Owner: core-product
Updated: 2026-03-19
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

- `g07.001` now freezes and partially realizes the canonical multichannel
  layout and channel-role contract on top of the current narrow `ChannelLayout`
  primitive: runtime topology, external-I/O, and plugin discovery or stage
  receipts now expose canonical layout, channel-role, and bus-intent meaning
  instead of leaving multichannel truth as raw count-only host inference
  - the shared consumer boundary is now closed through:
    - public runtime proof for multichannel topology and discovery receipts
    - stable local and server host-edge `supervisor_report()` proof
    - `signal-supervisor-tools --describe-multichannel-boundary`
    - `effigy acceptance:multichannel-boundary`
- `g07.002` now freezes and partially realizes the first sidechain and
  secondary-input routing boundary on top of that multichannel substrate:
  - sidechain source, target, attachment policy, and fallback outcome are now
    explicit Signal-owned routing vocabulary rather than host patching
    convention
  - `signal-runtime` now carries that meaning through:
    - `GraphNodeBufferContractProjection.secondary_input`
    - planned-node and execution-topology sidechain route receipts
    - plugin-chain stage sidechain receipts
    - offline render chain-dependency sidechain receipts
  - focused runtime proof now covers:
    - live topology and plugin-chain sidechain routing
    - offline render dependency preview sidechain alignment
  - the shared consumer boundary is now closed through:
    - public runtime proof for sidechain routing and fallback receipts
    - stable local and server host-edge `supervisor_report()` proof
    - `signal-supervisor-tools --describe-sidechain-boundary`
    - `effigy acceptance:sidechain-boundary`
  - broader multi-bus, complex plugin-I/O, and spatial routing behavior remain
    later work rather than implicit sidechain scope
- `g07.003` now freezes the first reusable multi-bus and auxiliary-topology
  contract, realizes it on runtime-owned execution surfaces, and proves the
  shared consumer seam on top
  of the closed multichannel and sidechain seams:
  - bus role, auxiliary path, connection identity, attachment class, and
    fallback outcome are now explicit Signal-owned routing vocabulary
  - execution topology, metering diagnostics, and offline render dependency
    preview now share typed multi-bus connection and auxiliary-path receipts
  - public runtime, stable host-edge, and supervisor-tools descriptor proof now
    close the bounded shared multi-bus consumer seam before later complex
    plugin-I/O or spatial depth
- `g07.004` now freezes the first reusable complex plugin-I/O contract on top
  of the closed multichannel, sidechain, and multi-bus routing seams:
  - plugin port class, complex plugin-I/O topology, multi-output instrument,
    and bus-capable FX are now explicit Signal-owned routing vocabulary
  - adapter capability and later runtime receipts now have one bounded target
    for richer plugin bus behavior rather than drifting into format-private pin
    naming
  - runtime realization, render/export depth, and the shared consumer boundary
    remain later `g07.004` work rather than implied by the contract freeze
- `g07.004` Batch 4.2 now materializes bounded complex plugin-I/O meaning on
  shared runtime surfaces:
  - discovered plugin-type receipts, format coverage, and capability coverage
    now carry typed complex plugin-I/O summaries instead of only raw feature
    and bus counts
  - plugin-chain stage snapshots and offline render dependency preview now
    preserve multi-output instrument and bus-capable FX topology through the
    same runtime-owned receipt family
  - VST3 and AU baseline fixtures now expose multi-output instrument and
    bus-capable FX shapes so the widened runtime surface is exercised against
    richer adapter catalogs before the public proof tranche
- `g07.004` Batch 4.3 now closes the bounded complex plugin-I/O consumer seam:
  - public runtime proof now covers complex discovery, live stage topology, and
    offline render dependency preview through one runtime-owned receipt family
  - stable local and server host edges now prove they forward the same
    multi-output instrument and bus-capable FX topology without adapter-local
    pin reconstruction
  - `signal-supervisor-tools` now exposes a machine-readable
    `signal.runtime.complex-io-boundary` descriptor and repo-owned acceptance
    task so the seam is inspectable rather than prose-only
- `g07.005` Batch 5.1 now freezes the first reusable spatial execution
  contract on top of those routing seams:
  - spatial adapter class, execution mode, target environment, control family,
    activation policy, and fallback outcome are now explicit Signal-owned
    vocabulary
  - later runtime execution, surround-bed expansion, and adapter breadth now
    have one bounded authority line instead of product-local pan policy or
    adapter-private renderer semantics
  - unsupported layouts, unsupported adapters, and target-environment gaps now
    must explain themselves through bounded fallback meaning rather than hidden
    host heuristics
- `g07.005` Batch 5.2 now materializes the first bounded runtime spatial path:
  - planned-node, execution-topology, and plugin-chain stage receipts now carry
    typed spatial execution summaries instead of leaving balance or fallback
    behavior implicit in raw stage lists
  - `RuntimeExecutionTopologySummary` now reports active, bypassed, and
    fallback spatial-node counts directly on the shared runtime observation
    surface
  - offline-render dependency preview now carries aligned spatial stage
    receipts so render planning no longer needs a separate spatial-only model
  - the current executable baseline is intentionally narrow and explicit:
    stereo `StereoBalance` stages realize bounded `BalanceGroups`, while
    non-stereo layouts surface `BypassSpatialProcessing` fallback
- `g07.005` Batch 5.3 now closes the shared consumer seam for that bounded
  spatial baseline:
  - public runtime proofs now verify execution-topology, plugin-chain, and
    offline-render spatial receipts without private helpers
  - both stable host edges now forward the same runtime-owned spatial execution
    and fallback vocabulary on supervisor export
  - `signal-supervisor-tools` now exposes a machine-readable
    `signal.runtime.spatial-boundary` descriptor and repo-owned acceptance task
    so the seam is inspectable rather than prose-only
- `g07.006` Batch 6.1 now freezes the next richer-spatial expansion boundary:
  - surround-bed class, object role, mix policy, render scope, and expanded
    fallback outcome are now explicit Signal-owned vocabulary
  - later surround or object runtime work now has one bounded authority line on
    top of the closed multichannel, multi-bus, complex plugin-I/O, and
    baseline spatial seams
  - richer immersive execution is still deferred, but the meaning it must obey
    is now explicit instead of living in product-local or renderer-private
    assumptions
- `g07.006` Batch 6.2 now materializes the first runtime-owned richer-spatial
  receipt layer on top of that contract:
  - the existing spatial execution summaries now also carry explicit bed class,
    object-role placeholder, object count, mix policy, render scope, and
    expanded fallback meaning rather than treating richer spatial depth as an
    implied property of channel layout alone
  - `RuntimeExecutionTopologySummary` now reports surround-bed, object-aware,
    and expanded-fallback spatial-node counts directly on the shared runtime
    observation surface
  - offline-render dependency preview now carries the same richer spatial
    counts and per-stage receipts so live execution and render planning stay on
    one bounded model
  - the current executable path is still intentionally narrow and explicit:
    stereo `StereoBalance` stages realize `StereoBed` plus `BedOnly`, while
    canonical surround stages surface `CanonicalSurroundBed` plus
    `CollapseToBaselineSpatial` instead of silent non-stereo bypass
- `g07.006` Batch 6.3 now closes the shared consumer seam for that richer
  spatial substrate:
  - public runtime proofs now verify surround-bed, mix-policy, render-scope,
    and expanded-fallback receipts through observation, supervisor, and
    offline-render preview surfaces without private helpers
  - both stable host edges now forward the same richer spatial model on
    supervisor export without host-local speaker or renderer reinterpretation
  - `signal-supervisor-tools` now keeps the existing
    `signal.runtime.spatial-boundary` descriptor aligned to the richer
    `g07.006` contract instead of the earlier baseline-only spatial contract
- `g07.007` Batch 7.1 now freezes the first LV2 adapter alignment boundary:
  - LV2 discovery, lifecycle, and Linux-native support are now mapped onto the
    existing backend-neutral plugin and runtime contract family instead of
    being left as Linux-only host intent
  - the current gaps are now explicit: no shared LV2 backend identity, no Rust
    LV2 adapter realization, and no runtime-owned Linux-native scan or load
    receipts exist yet
  - later LV2 runtime work now has one bounded authority line to extend
    without reopening host-local Linux plugin ownership or adapter-private
    lifecycle semantics
- `g07.007` Batch 7.2 now realizes the first Linux-native LV2 adapter slice:
  - `signal-plugin` now exposes `PluginFormat::Lv2`, and
    `signal-plugin-lv2` now owns bounded LV2 scan-root, URI, manifest-path,
    and session-planning fixtures instead of leaving LV2 as contract-only
    breadth
  - the server host now feeds LV2 discovery and sandbox ensure through the
    same runtime-owned discovered-type, lifecycle, instance-state, transport,
    and parity receipts already used by the other plugin formats
  - Linux-only LV2 platform scope is now explicit on runtime-owned parity and
    platform-coverage surfaces rather than implied by Linux roadmap intent
- `g07.007` Batch 7.3 now closes the shared LV2 proof seam:
  - public runtime proofs now verify Linux-native LV2 discovery, lifecycle,
    transport, and platform-scope truth through shared runtime reports
  - the stable server host edge now forwards the same LV2 truth on supervisor
    export without adapter-local or host-local Linux reconstruction
  - `signal-supervisor-tools` now exposes `signal.runtime.lv2-boundary`, and
    Effigy now owns an LV2 boundary acceptance lane for downstream consumers
- `g07.008` Batch 8.1 now freezes the bounded Linux cross-adapter plugin parity
  and sandbox-policy contract:
  - CLAP, VST3, and LV2 now share one Linux-facing parity vocabulary for
    portable, guarded, adapter-private, and unsupported behavior
  - Linux sandbox and placement-policy meaning is now explicitly reused from
    the shared runtime-owned continuity and shared-sandbox contract rather than
    a Linux-only wrapper taxonomy
  - later runtime work now has one bounded Linux parity target for lifecycle,
    render, failure, and placement receipts instead of separate adapter claims
- `g07.008` Batch 8.2 now realizes that bounded Linux parity contract on
  runtime-owned receipt surfaces:
  - `RuntimePluginFormatPlatformCoverageRecord` and
    `RuntimePluginFormatParityRecord` now carry Linux-specific parity band,
    Linux support, preferred sandbox outcome, strict-sandbox default,
    render-capable type counts, and restart or rebindability counts
  - `RuntimePluginDiscoverySnapshot`, `RuntimePluginScanReceipt`, and
    `RuntimePluginLifecycleSnapshot` now share the same widened Linux parity
    record family instead of forcing Linux consumers to infer policy from
    broader cross-platform parity bands
  - the Linux server host now feeds that widened parity surface directly on the
    same runtime-owned discovery and lifecycle path for VST3 and LV2
- `g07.008` Batch 8.3 now closes the shared Linux parity proof seam:
  - public runtime proofs now verify Linux-specific parity band, Linux support,
    preferred sandbox outcome, strict-sandbox default, and restart or failure
    posture through shared runtime observation and supervisor reports
  - the stable server host edge now forwards the same CLAP, VST3, and LV2
    Linux parity truth without host-local Linux portability matrices
  - `signal-supervisor-tools` now exposes
    `signal.runtime.linux-plugin-parity-boundary`, and Effigy now owns a Linux
    parity acceptance lane for downstream consumers
- `g07.009` Batch 9.1 now freezes the bounded Linux hardware backend
  portability contract:
  - ALSA, JACK, and PipeWire now have one explicit Signal-owned portability
    vocabulary instead of remaining future backend-private breadth
  - Linux backend lifecycle, supervision, clocking, and endpoint interpretation
    are now explicitly required to compose through the shared hardware,
    supervision, and clock-domain contracts
  - later backend baseline work now has one bounded Linux hardware contract
    target instead of separate backend narratives
- `g07.010` Batch 10.1 now freezes the bounded Linux backend clocking, duplex,
  and endpoint-topology parity contract:
  - ALSA, JACK, and PipeWire now have one explicit Linux-facing parity target
    for clocking, duplex, and endpoint-topology meaning
  - Linux backend identity remains anchored in the closed portability contract,
    while clocking and topology parity are now required to compose through the
    shared drift, discontinuity, duplex-mismatch, and supervision boundaries
  - backend-native daemon, graph, and node detail remains advisory until later
    runtime receipt work promotes it
- `g07.010` Batch 10.2 now materializes the first runtime-owned Linux backend
  parity receipt depth:
  - `RuntimeHostClockingSummary` and `RuntimeExternalIoSnapshot` now carry
    explicit Linux-specific clocking, duplex, and endpoint-topology parity
    classification alongside the generic hardware and clocking fields
  - ALSA-style steady same-clock paths now classify as portable, guarded
    aggregate or recovering Linux paths stay explicit, and non-Linux or
    unavailable host contexts export typed unsupported parity instead of
    implied gaps
  - local-host and server-host shared reports now forward the same Linux
    parity vocabulary, which narrows the remaining work to the public proof
    seam
- `g07.010` Batch 10.3 now closes the bounded Linux backend clock-topology
  consumer seam:
  - public runtime proofs now verify ALSA, JACK, PipeWire, non-Linux, and
    unavailable host contexts keep Linux-specific clocking, duplex, and
    endpoint-topology parity consumable through shared observation and
    supervisor receipts
  - the stable local and server host edges now forward explicit unsupported or
    unavailable Linux parity on shared export instead of falling back to
    backend-private Linux capability matrices
  - `signal-supervisor-tools` now exposes
    `signal.runtime.linux-backend-clock-topology-boundary`, and Effigy now
    owns `acceptance:linux-backend-clock-topology-boundary` as the repo-owned
    rerun lane
- `g07.011` Batch 11.1 now freezes the bounded external MIDI endpoint and
  device-identity contract:
  - external MIDI device identity, endpoint identity, endpoint graph, bounded
    capability, lifecycle, and route meaning now have one explicit
    runtime-owned target instead of product-local browser or patchbay models
  - generic MIDI event meaning remains anchored in the closed `g06.012`
    contract, which prevents later endpoint work from reopening a second
    transport-private event vocabulary
  - later runtime baseline work now has one fixed contract target for external
    MIDI endpoint receipts rather than backend-private patchbay semantics
- `g07.011` Batch 11.2 now materializes the first runtime-owned external MIDI
  endpoint baseline:
  - `RuntimeObservationReport` and `RuntimeSupervisorReport` now carry typed
    external MIDI graph, device, endpoint, capability, and route receipts
    instead of leaving endpoint truth implicit or host-private
  - runtime capture now defaults to explicit `Unavailable` external MIDI state,
    while local and server host edges both project the same `Empty` graph
    baseline through shared runtime-owned export
  - compact, multiline, and JSON report rendering now all carry the same
    bounded external MIDI receipt family, which narrows the remaining work to
    the public proof seam
- `g07.011` Batch 11.3 now closes the bounded external MIDI consumer seam:
  - public runtime proof now keeps typed `Unavailable` and `Empty` external
    MIDI endpoint graph state consumable through shared observation and
    supervisor receipts
  - both stable host edges now prove they forward the same runtime-owned empty
    external MIDI graph baseline instead of host-local MIDI device
    reconstruction
  - `signal-supervisor-tools` now exposes
    `signal.runtime.external-midi-boundary`, and Effigy now owns
    `acceptance:external-midi-boundary` as the repo-owned rerun lane
- `g07.012` Batch 12.1 now freezes the widened MIDI 2.0 and controller-
  expression contract:
  - richer controller-expression, MPE posture, MIDI 2.0 posture, and guarded
    widening now have one explicit runtime-owned contract target instead of
    adapter-private packet models becoming the consumer boundary
  - generic event meaning from `g06.012` and external MIDI endpoint meaning
    from `g07.011` remain the anchors, which prevents later widening from
    reopening a second event or device shell
  - later runtime, plugin, and hardware work now has one fixed widened
    expressive-event contract target rather than speculative controller-depth
    drift
- `g07.012` Batch 12.2 now materializes the first widened controller-
  expression receipt family:
  - `signal-plugin::EventPacketSummary` now breaks widened note expression into
    pressure, timbre, and tuning families instead of one opaque widened count
  - `RuntimePluginEventSnapshot` now carries those richer family totals plus
    runtime-owned `MPE` and `MIDI 2.0` posture derived from shared event
    evidence
  - `RuntimeExternalMidiEndpointCapabilitySummary` now exposes explicit
    guarded-or-unsupported widened capability posture for richer expression
    families on the external MIDI hardware boundary
- `g07.012` Batch 12.3 now closes the widened controller-expression proof
  seam:
  - public runtime now proves widened note-expression family totals, `MPE`
    posture, `MIDI 2.0` posture, and bounded external-device controller-
    expression capability posture through shared runtime DTOs
  - both stable host edges now prove they forward the same widened
    controller-expression truth instead of host-private packet or capability
    reconstruction
  - `signal-supervisor-tools` now exposes
    `signal.runtime.controller-expression-boundary`, and Effigy now owns
    `acceptance:controller-expression-boundary` as the repo-owned rerun lane
- `g07.013` Batch 13.1 now freezes the bounded control-surface transport and
  feedback contract:
  - control-surface device identity, transport posture, feedback readiness,
    mapping posture, and bounded capability meaning now have one explicit
    runtime-owned contract target instead of host-local controller integration
    logic
  - external MIDI endpoint meaning from `g07.011` and widened controller-
    expression meaning from `g07.012` remain the anchors, which prevents later
    control-surface work from reopening a second device or event shell
  - later runtime work now has one fixed control-surface contract target
    before mapping, feedback, and extensibility depth widens
- `g07.013` Batch 13.2 now materializes the first runtime-owned control-surface
  baseline:
  - `RuntimeControlSurfaceSnapshot` and per-device descriptors now derive
    transport posture, mapping posture, feedback readiness, and widened-
    expression capability directly from the closed external MIDI endpoint graph
  - observation, supervisor, and both stable host-edge report paths now carry
    the same control-surface snapshot family, including explicit unavailable,
    empty, and guarded outcomes
  - this keeps controller transport and feedback meaning inside shared runtime
    receipts instead of host-local controller-policy reconstruction, while
    leaving the public proof seam for Batch 13.3
- `g07.013` Batch 13.3 now closes the bounded control-surface proof seam:
  - public runtime now proves `RuntimeControlSurfaceSnapshot` remains
    consumable through shared runtime reports without host-local controller
    policy
  - both stable host edges now prove they forward the same control-surface
    transport, mapping-posture, feedback-readiness, and capability truth
  - `signal-supervisor-tools` now exposes
    `signal.runtime.control-surface-boundary`, and Effigy now owns
    `acceptance:control-surface-boundary` as the repo-owned rerun lane
- `g07.014` Batch 14.1 now freezes the bounded advanced-hardware extensibility
  and scripting-safe device-policy contract:
  - advanced device capability classes, guarded feedback channels, and typed
    device action classes now have one explicit runtime-owned contract target
    instead of host-local hardware exception handling
  - scripting-safe policy posture is now explicitly runtime-owned and separates
    portable, guarded, context-only, denied, and unsupported outcomes without
    absorbing product-local controller setup or scripting workflow
  - external MIDI endpoint and control-surface meaning remain the anchors,
    which prevents later advanced hardware work from reopening a second device
    or scripting shell
- `g07.014` Batch 14.2 now materializes the first runtime-owned
  advanced-hardware receipt family:
  - `RuntimeAdvancedHardwareSnapshot` now derives advanced-hardware graph
    state, scripting-safe device-policy posture, guarded feedback-channel
    posture, and typed action classes from the closed control-surface baseline
  - observation, supervisor, and stable host-edge report paths now carry
    explicit unavailable, empty, guarded, and ready advanced-hardware outcomes
    instead of host-local controller-policy reconstruction
  - the baseline stays intentionally bounded to guarded display and navigation
    posture while richer vendor protocols, motor or haptic depth, and
    executable scripting remain deferred
- `g07.014` Batch 14.3 now closes the bounded advanced-hardware proof seam:
  - public runtime now proves `RuntimeAdvancedHardwareSnapshot` remains
    consumable through shared runtime reports without host-local hardware or
    controller-policy reconstruction
  - both stable host edges now prove they forward the same advanced-hardware
    graph state, scripting-safe device-policy posture, guarded feedback-channel
    posture, and typed action-class truth
  - `signal-supervisor-tools` now exposes
    `signal.runtime.advanced-hardware-boundary`, and Effigy now owns
    `acceptance:advanced-hardware-boundary` as the repo-owned rerun lane
- `g07.015` Batch 15.3 now closes the bounded sample-domain stretch consumer
  seam:
  - public runtime now proves `RuntimeStretchEngineSnapshot` remains
    consumable through shared runtime reports, clip-render receipts, and
    offline-render preview without host-local transform reconstruction
  - both stable host edges now prove they forward the same stretch-engine
    class, readiness, degraded-state, and fallback truth through supervisor
    export
  - `signal-supervisor-tools` now exposes
    `signal.runtime.stretch-boundary`, and Effigy now owns
    `acceptance:stretch-boundary` as the repo-owned rerun lane
- `g07.016` Batch 16.1 now freezes the bounded warp-marker, transient-anchor,
  and tempo-assist analysis contract:
  - marker, anchor, tempo-assist, readiness, degraded-state, and invalidation
    meaning must now widen from the closed media-service, analysis-metadata,
    and sample-domain stretch-engine seams instead of host-local marker tools
  - later artifact-cache, preview, and audition work is now forced to deepen
    one shared runtime-owned analysis boundary instead of reopening a second
    transform-analysis shell
- `g07.016` Batch 16.2 now materializes the first bounded runtime-owned
  marker-analysis receipt family:
  - `RuntimeMarkerAnalysisSnapshot` now derives warp-marker counts,
    transient-anchor counts, tempo-assist posture, readiness, and invalidation
    from shared clip-processing, stretch, warp, and media-library truth
  - runtime observation, supervisor export, and both stable host edges now
    surface the same marker-analysis receipts instead of reconstructing
    host-local stretch-analysis state
  - the realized baseline stays bounded to current reusable analysis
    descriptors rather than claiming a fuller editor-grade marker engine
- `g07.016` Batch 16.3 now closes the bounded marker-analysis consumer seam:
  - public runtime now proves `RuntimeMarkerAnalysisSnapshot` remains
    consumable through shared runtime reports without host-local
    stretch-analysis reconstruction
  - both stable host edges now prove they forward the same runtime-owned
    warp-marker, transient-anchor, tempo-assist, readiness, and invalidation
    receipts through supervisor export
  - `signal-supervisor-tools` now exposes
    `signal.runtime.marker-analysis-boundary`, and Effigy now owns
    `acceptance:marker-analysis-boundary` as the repo-owned rerun lane
- `g07.017` Batch 17.1 now freezes the bounded post-warp render and
  transform-artifact contract:
  - transform-artifact identity, readiness, invalidation, reuse, and degraded
    posture must now widen from the closed media, stretch-engine, and
    marker-analysis seams instead of host-local preview caches
  - later preview and audition work is now forced to deepen one shared
    runtime-owned transform-artifact boundary instead of reopening a second
    preview-cache shell
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
- `effigy acceptance:linux-plugin-parity-boundary`
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
- `docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md`
- `crates/signal-plugin-vst3/src/lib.rs`
- `crates/signal-supervisor-tools/src/main.rs`
- `effigy.toml`
- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-runtime/src/lib.rs`
- `crates/signal-runtime/tests/public_contract_boundary.rs`
- `crates/signal-host-local/tests/public_host_edge_boundary.rs`
- `crates/signal-host-server/tests/public_host_edge_boundary.rs`

## Batch 17.2 Outcome

- `signal-runtime` now owns `RuntimeTransformArtifactSnapshot` and
  `RuntimeTransformArtifactClipSnapshot`, derived from shared clip-processing,
  stretch-engine, marker-analysis, and media-pipeline truth instead of
  preview-cache or export-local inference.
- the same transform-artifact receipt family now flows through runtime
  observation, supervisor export, clip-render results, offline-render preview,
  and stable host-edge JSON.
- the bounded artifact baseline now exposes readiness, invalidation,
  cached-media readiness, and reuse posture directly for later cache and
  audition depth.

## Batch 17.3 Outcome

- public runtime, stable host-edge, and `signal-supervisor-tools` surfaces now
  prove transform-artifact readiness, invalidation, cached-media readiness,
  and reuse remain consumable without host-local preview-cache reconstruction.
- Effigy now owns `acceptance:transform-artifact-boundary` as the repo-owned
  rerun lane for the bounded post-warp artifact seam.

## Batch 18.1 Outcome

- Signal now has a frozen runtime-owned contract for low-latency audition,
  scrub preview, preview service class, readiness, degraded state, fallback,
  and artifact alignment on top of the closed stretch, marker-analysis, and
  transform-artifact seams.
- later preview-service work is now forced to deepen one shared runtime-owned
  preview vocabulary instead of reopening host-local preview players,
  product-local browser shells, or private scrub transform models.

## Batch 18.2 Outcome

- `signal-runtime` now owns `RuntimePreviewTransformServiceSnapshot`, derived
  directly from the closed media-service, stretch-engine, marker-analysis, and
  transform-artifact seams instead of host-local preview playback state.
- runtime observation, supervisor export, clip-render results, offline render
  contract preview, and both stable host-edge JSON paths now expose the same
  preview service class, readiness, degraded state, fallback, active audition,
  and scrub-supported receipts.

## Batch 18.3 Outcome

- public runtime, both stable host edges, and `signal-supervisor-tools` now
  prove one shared `signal.runtime.preview-transform-boundary` for preview
  readiness, degraded-state, fallback, active audition, and scrub-supported
  truth.
- Effigy now owns `acceptance:preview-transform-boundary` as the repo-owned
  rerun lane for the bounded preview-transform seam.

## Batch 19.1 Outcome

- Signal now has a frozen integrated acceptance contract for the widened
  multichannel, Linux, controller, and stretch surfaces instead of only
  milestone-local boundary reruns.
- later `g07.019` work is now constrained to one shared required, advisory,
  and deferred policy for grouped acceptance descriptors and lanes.

## Batch 19.2 Outcome

- `signal-supervisor-tools` now exposes the first grouped `g07` acceptance-lane
  descriptor, and Effigy now owns a repo-owned rerun lane across the required
  routing, Linux, controller, and stretch families.
- the acceptance surface is now runnable as one grouped lane instead of only a
  roadmap promise over separate boundary tasks.

## Batch 19.3 Outcome

- the grouped `g07` acceptance lane now proves one `signal.supervisor.export`
  payload can carry routing, Linux backend, control-surface or advanced-hardware,
  and stretch or preview receipts together instead of only replaying the
  component boundary tasks.
- Effigy now keeps that cross-family export proof inside the repo-owned
  `acceptance:g07-integrated-acceptance-lane` rerun task, so later closeout
  work can build on one coherent evidence surface.

## Batch 20.1 Outcome

- `g07` closeout now has a frozen repo-owned policy in
  `docs/contracts/051-generation-closeout-and-loophole-feature-readiness-gate-contract.md`
  instead of an open-ended final review shape.
- the final gate is now constrained to one authority line: the closed routing,
  Linux, controller, and stretch seams, the grouped integrated acceptance lane,
  one future machine-readable closeout descriptor, and one explicit
  Loophole-facing readiness verdict.

## Batch 20.2 Outcome

- `signal-supervisor-tools` now emits one machine-readable `g07` closeout
  descriptor through `--describe-generation-closeout`, with `g07`-specific
  contract, roadmap, grouped-acceptance, provisional readiness, and residual
  risk state.
- Effigy now owns `acceptance:g07-closeout`, making the closeout gate runnable
  on top of the grouped `g07` acceptance lane instead of leaving Batch 20.3 to
  invent its own validation surface.
- the final Loophole-facing readiness verdict and any next-generation or
  backlog decision remain intentionally deferred, but they now have one typed
  gate to review instead of a prose-only milestone close.

## Batch 20.3 Outcome

- the `g07` closeout descriptor now records a real promotion verdict:
  `promote-g08`, with all `g07` readiness areas resolved to
  `sufficient-for-promotion`
- the bounded `g07` gate is now enough to treat routing, Linux breadth,
  controller substrate, and sample-domain transform services as closed reusable
  Signal substrate instead of an active blocker
- richer Linux live ownership, immersive routing, vendor-protocol hardware,
  and preview-browser workflow depth stay explicit as `g08` scope rather than
  silently blocking `g07` closeout

## g08.001 Batch 1.1 Outcome

- `g08` now has a frozen live Linux backend ownership contract in
  `docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`
  instead of relying on the bounded portability and parity seams alone
- ALSA, JACK, and PipeWire live attach, running, recovery, release, and
  unavailable posture are now required to compose through shared hardware,
  supervision, and external-I/O receipts instead of backend-private daemon or
  graph lifecycle shells
- backend-native graph, transport, node, and session-manager detail remains
  explicitly private until later `g08` promotion

## g08.001 Batch 1.2 Outcome

- `signal-runtime` now owns `RuntimeLinuxBackendSessionSnapshot` plus typed
  ownership, lifecycle, device-claim, role, and fallback state for live Linux
  backend sessions
- the first derivation path composes from `RuntimeHostIoSummary`, keeping live
  Linux ownership meaning runtime-owned instead of daemon-local or host-local
- host-local now exports an explicit `NotLinux` answer, while server-host
  exports a bounded simulated PipeWire backend-managed session baseline on the
  same shared runtime seam

## g08.001 Batch 1.3 Outcome

- public runtime, both stable host edges, and supervisor-tools now prove the
  live Linux backend ownership seam through one shared
  `RuntimeLinuxBackendSessionSnapshot` contract instead of backend-private
  Linux session stories
- local host proves explicit `NotLinux` export, while server host proves the
  bounded PipeWire-style live-session baseline stays runtime-owned
- Effigy now owns `acceptance:linux-live-ownership-boundary` so this seam can
  be rerun as a repo-owned contract before deeper JACK and backend-native work
  widens

## g08.002 Batch 2.1 Outcome

- `g08` now has a frozen JACK transport, graph, and backend-native
  coordination contract in
  `docs/contracts/053-jack-transport-graph-and-backend-native-coordination-contract.md`
  instead of relying on the closed live Linux ownership seam alone
- JACK transport posture, graph attachment, client role, and guarded
  coordination are now required to compose through shared runtime, hardware,
  and supervision receipts instead of host-private callback policy
- JACK callback-thread, daemon, port-ID, and session-manager details remain
  explicitly private until later `g08` promotion

## g08.002 Batch 2.2 Outcome

- `signal-runtime` now owns a bounded `RuntimeJackCoordinationSnapshot`
  derived from shared Linux host-I/O and transport-session evidence instead of
  host-private JACK callback policy
- JACK transport posture, graph-state, client-role, and guarded-coordination
  answers now stay typed on shared runtime, observation, and supervisor
  surfaces
- stable host edges export explicit bounded answers on that same seam:
  `NotJack` on local host and a simulated guarded JACK graph baseline on
  server host
## g08.002 Batch 2.3 Outcome

- public runtime now proves JACK transport posture, graph coordination,
  client role, and guarded state through one downstream-style observation and
  supervisor boundary
- stable host edges now keep that seam explicit:
  `NotJack` on local host and a bounded guarded JACK graph baseline on server
  host
- supervisor-tools and Effigy now expose
  `signal.runtime.jack-coordination-boundary` plus
  `acceptance:jack-coordination-boundary` as the repo-owned consumer-proof
  seam

## g08.003 Batch 3.3 Outcome

- `signal-runtime` now exports `RuntimePipeWireAlsaParitySnapshot` as the
  bounded shared receipt for PipeWire and ALSA session-role, device-claim,
  stream-policy, and guarded parity
- stable host edges are aligned to the same receipt family:
  - local host proves `NotPipeWireOrAlsa`
  - server host proves backend-managed PipeWire policy with clock-guarded
    parity
- `signal-supervisor-tools` now exposes
  `signal.runtime.pipewire-alsa-parity-boundary` as the machine-readable
  consumer descriptor for that receipt family
- `effigy acceptance:pipewire-alsa-parity-boundary` now composes the public
  runtime proof, stable host-edge proofs, and descriptor proof into one
  repo-owned acceptance seam
- `g08.003` is complete, so later Linux workflow and acceptance milestones can
  build on one explicit PipeWire and ALSA authority line instead of reopening
  host-local parity reconstruction

## g08.004 Batch 4.1 Outcome

- `g08` now has a frozen LV2 worker, URID, patch, and extension-negotiation
  contract in
  `docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md`
  instead of leaving that richer LV2 depth as deferred prose on the earlier
  LV2 baseline
- worker posture, URID negotiation posture, patch exchange posture, and
  extension-negotiation summary are now required to compose through shared
  runtime-owned receipts instead of host-local feature tables or adapter-only
  negotiation logs
- full atom-schema, UI, custom extension, and Linux backend session detail
  remain explicitly private or deferred until later promotion

## g08.004 Batch 4.2 Outcome

- `signal-runtime` now exports one `RuntimeLv2ExtensionSnapshot` surface for
  worker posture, URID negotiation posture, patch exchange posture, and
  extension-negotiation state instead of leaving that meaning in adapter-only
  feature tables
- the LV2 extension seam now composes from runtime-owned discovery and plugin
  lifecycle truth, so negotiated, guarded, and unavailable outcomes stay typed
  across shared observation and supervisor export
- stable host-edge surfaces now expose the same runtime-owned LV2 extension
  receipt family instead of reconstructing LV2 extension support from
  host-local summaries

## g08.004 Batch 4.3 Outcome

- the existing `signal.runtime.lv2-boundary` consumer seam now points at the
  LV2 worker, URID, patch, and extension-negotiation contract instead of the
  earlier baseline-only LV2 contract
- the repo-owned acceptance lane now requires public runtime proof plus stable
  local and server host-edge proofs for the same runtime-owned LV2 extension
  snapshot
- supervisor-side machine-readable boundary output now describes the bounded
  LV2 extension seam directly, so consumers can inspect the proof surface
  without adapter-private reconstruction

## g08.005 Batch 5.1 Outcome

- `g08` now has a frozen complex plugin pin-matrix and dynamic
  bus-negotiation contract in
  `docs/contracts/056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md`
  instead of leaving that richer plugin-routing depth as deferred prose on the
  earlier complex-I/O baseline
- pin-group identity, pin-matrix posture, dynamic bus-negotiation posture, and
  negotiation fallback outcome are now required to compose through shared
  runtime-owned receipts instead of host-local bus rules or adapter-private
  pin graphs
- full format-specific pin schemas, product pin-matrix UX, and richer runtime
  execution receipts remain explicitly deferred until later promotion

## g08.005 Batch 5.2 Outcome

- `signal-runtime` now exports one `RuntimePluginPinMatrixSnapshot` surface for
  pin-group identity, pin-matrix posture, dynamic bus-negotiation posture, and
  bounded fallback outcome instead of leaving that richer routing meaning in
  adapter-private port graphs
- the pin-matrix seam now composes from runtime-owned complex-I/O discovery,
  sandbox lifecycle, and plugin-chain stage truth, so declared, negotiated,
  guarded, and unavailable outcomes stay typed across shared observation and
  supervisor export
- stable host-edge surfaces now expose the same runtime-owned pin-matrix
  receipt family instead of reconstructing complex bus activation posture from
  host-local plugin detail

## g08.005 Batch 5.3 Outcome

- the existing `signal.runtime.complex-io-boundary` consumer seam now points at
  the pin-matrix and dynamic bus-negotiation contract instead of the earlier
  baseline-only complex-I/O contract
- the machine-readable supervisor boundary now describes both the prior
  complex-I/O receipts and the new `plugin_pin_matrix_snapshot` surface as one
  bounded shared proof seam
- the repo-owned acceptance lane continues to reuse the focused runtime and
  host-edge proofs, but now closes the widened plugin-routing consumer seam
  without plugin-format-specific negotiation policy

## g08.006 Batch 6.1 Outcome

- `g08` now has a frozen immersive object-rendering and room-policy contract in
  `docs/contracts/057-immersive-object-rendering-and-room-policy-substrate-contract.md`
  instead of leaving immersive room-policy truth as deferred prose underneath
  the earlier spatial and richer-spatial seams
- immersive object-rendering posture, room-policy class, room-policy
  authority, and immersive room outcome are now required to compose through
  shared runtime-owned meaning instead of renderer-private room rules or
  host-local deployment heuristics
- speaker deployment, fold-down, monitoring-scene depth, renderer-capability
  negotiation, and immersive export packaging remain explicitly deferred until
  later `g08` milestones

## g08.006 Batch 6.2 Outcome

- `signal-runtime` now carries one bounded `immersive_room_policy` summary on
  the existing richer-spatial execution surface instead of introducing a second
  immersive report family or renderer-private room-policy model
- execution topology, plugin-chain stages, and offline-render dependency
  preview now all expose aggregate immersive room-policy counts, so later
  immersive monitoring and export work can build on one runtime-owned
  inspection seam
- the current runtime-owned baseline stays explicit: canonical surround
  fallback paths now surface `FallbackRoom` plus `BypassRoomPolicy`, while
  stereo-only paths remain outside the immersive room-policy seam until later
  renderer-backed depth exists

## g08.006 Batch 6.3 Outcome

- the existing `signal.runtime.spatial-boundary` consumer seam now points at
  the immersive room-policy contract instead of the earlier richer-spatial-only
  contract
- the machine-readable supervisor boundary now describes immersive room-policy
  topology, plugin-chain, and offline-render preview anchors as one bounded
  shared proof seam
- the repo-owned acceptance lane continues to reuse the focused runtime and
  host-edge proofs, but now closes the widened immersive room-policy consumer
  seam without a renderer-private room-policy shell

## g08.007 Batch 7.3 Outcome

- the existing `signal.runtime.spatial-boundary` now points at the speaker
  deployment and monitoring contract instead of stopping at the earlier
  immersive room-policy seam
- the machine-readable supervisor boundary now describes deployment-aware,
  folded-down, and fallback-monitoring topology, stage, and render-preview
  anchors alongside `deployment_monitoring` on shared runtime receipts
- the repo-owned acceptance lane continues to reuse the focused runtime and
  host-edge proofs, but now closes the bounded deployment and monitoring
  consumer seam without a renderer-private monitoring shell

## g08.008 Batch 8.2 Outcome

- `signal-runtime` now carries bounded renderer-capability negotiation and
  immersive-export posture on the shared spatial execution seam through
  `renderer_export`
- execution topology and offline render preview now count
  renderer-capability, negotiated-renderer, immersive-export, and
  fallback-export work directly from runtime-owned receipts
- the focused public runtime and stable host-edge proofs now assert the same
  fallback renderer negotiation and immersive export answers instead of
  relying on renderer-private capability tables or host-local export glue

## g08.008 Batch 8.3 Outcome

- the existing `signal.runtime.spatial-boundary` now points at the
  renderer-capability and immersive-export contract instead of stopping at the
  earlier deployment and monitoring seam
- the machine-readable supervisor boundary now describes
  `spatial_execution.renderer_export` plus renderer-capability and
  immersive-export topology and render-preview anchors as one bounded shared
  proof seam
- the repo-owned acceptance lane continues to reuse the focused runtime and
  host-edge proofs, but now closes the bounded renderer-capability and
  immersive-export consumer seam without a renderer-private export shell

## g08.009 Batch 9.1 Outcome

- `g08` now has a frozen advanced control-surface display, motor, and haptic
  transport contract in
  `docs/contracts/060-advanced-control-surface-display-motor-and-haptic-transport-contract.md`
  instead of leaving richer control-surface feedback depth as deferred prose
  under the earlier control-surface and advanced-hardware seams
- display transport posture, display content class, motor transport posture,
  haptic transport posture, feedback authority, and feedback outcome are now
  required to compose through shared runtime-owned meaning instead of
  vendor-private payload schemas, host-local feedback bridges, or product-local
  controller UX
- runtime receipts, supervisor proof, and stable host-edge export remain
  explicitly deferred until later `g08.009` batches

## g08.009 Batch 9.2 Outcome

- `signal-runtime` now carries typed display posture, display content class,
  motor posture, haptic posture, feedback authority, and feedback outcome on
  the existing advanced-hardware seam instead of leaving richer feedback depth
  at the contract layer only
- the advanced-hardware snapshot now exposes aggregate display, motor, and
  haptic transport counts, so consumers do not need to reconstruct richer
  feedback truth from action-class flags alone
- the focused public runtime and stable host-edge proofs now assert the same
  bounded guarded-display baseline instead of relying on vendor-private payload
  schemas or host-local feedback bridges

## g08.009 Batch 9.3 Outcome

- the existing `signal.runtime.advanced-hardware-boundary` now points at the
  advanced control-surface display, motor, and haptic transport contract
  instead of stopping at the earlier advanced-hardware baseline seam
- the machine-readable supervisor boundary now describes display, motor, and
  haptic transport counts plus device-level posture and bounded feedback
  outcome anchors as one bounded shared proof seam
- the repo-owned acceptance lane continues to reuse the focused runtime and
  host-edge proofs, but now closes the bounded advanced control-feedback
  consumer seam without a device-private feedback shell

## g08.010 Batch 10.1 Outcome

- `g08` now has a frozen control-surface scene mapping, feedback-page, and
  safe action graph contract in
  `docs/contracts/061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md`
  instead of leaving richer controller workflow depth as deferred prose under
  the earlier control-surface and advanced-feedback seams
- scene-mapping posture, feedback-page posture, feedback-page class, safe
  action graph posture, action authority, and safe action outcome are now
  required to compose through shared runtime-owned meaning instead of
  controller-page assumptions, host-local scene ledgers, or unsafe device
  scripts
- runtime receipts, supervisor proof, and stable host-edge export remain
  explicitly deferred until later `g08.010` batches

## Next Task

Continue `g08.013` with Batch 13.3 by proving the widened persistence-policy
seam through shared runtime, supervisor, and stable host-edge surfaces
without introducing a browser-local storage ledger or host-local cache-policy
shell.

## g08.011 Batch 11.1 Outcome

- `g08` now has a frozen preview-device contract in
  `docs/contracts/062-preview-output-routing-audition-sink-and-low-latency-device-policy-contract.md`
  instead of leaving preview-device routing as deferred prose under the older
  preview-transform and external-I/O seams
- preview-output routing, audition-sink ownership, and low-latency device
  policy are now required to compose through the closed preview-transform,
  media-service, external-I/O, controller, and advanced-hardware seams rather
  than browser-local preview buses or host-local device picks
- runtime receipts, supervisor proof, and stable host-edge export remain
  explicitly deferred until Batch 11.2 and Batch 11.3

## g08.011 Batch 11.2 Outcome

- `signal-runtime` now widens the existing preview-transform seam with bounded
  preview-output routing, audition-sink, and low-latency device-policy truth
  instead of opening a second preview delivery report family
- `RuntimePreviewTransformServiceSnapshot` now carries a typed
  `preview_device_policy` summary covering routing posture, sink class,
  authority, policy class, and policy outcome
- public runtime and stable host-edge proofs now consume the same runtime-
  owned preview-device truth, so later supervisor proof can widen the current
  preview-transform boundary instead of introducing a host-local route or
  device-picker shell

## g08.011 Batch 11.3 Outcome

- the existing `signal.runtime.preview-transform-boundary` now points at
  `docs/contracts/062-preview-output-routing-audition-sink-and-low-latency-device-policy-contract.md`
  and describes bounded preview-device policy on observation, supervisor,
  render-preview, and offline-preview surfaces
- the machine-readable supervisor boundary now closes the bounded preview-route
  and audition-sink consumer seam through the existing public runtime and
  stable host-edge proof spine instead of introducing a preview-device-only
  acceptance lane
- `g08.011` is now complete, and the next `g08` queue is preview-browser
  queueing, media audition orchestration, and transform scheduling depth

## g08.012 Batch 12.1 Outcome

- `g08` now has a frozen preview-workflow contract in
  `docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md`
  instead of leaving preview-browser queueing, media audition orchestration,
  and transform scheduling as deferred prose under the earlier preview seams
- preview-browser queueing, media audition orchestration, and transform
  scheduling are now required to compose through the closed media-service,
  preview-transform, and preview-device seams rather than browser-local
  queues, editor-local audition schedulers, or app-specific transform timing
  shells
- runtime receipts, supervisor proof, and stable host-edge export remain
  explicitly deferred until later `g08.012` batches

## g08.012 Batch 12.2 Outcome

- `signal-runtime` now widens the existing preview-transform seam with bounded
  preview-browser queue, media audition orchestration, and transform-
  scheduling truth instead of opening a second preview-workflow report family
- `RuntimePreviewTransformServiceSnapshot` now carries a typed
  `preview_workflow` summary covering queue posture, queue class, queue
  outcome, audition posture, audition authority, continuity outcome, and
  transform-scheduling posture, authority, and outcome
- public runtime and stable host-edge proofs now consume the same runtime-
  owned preview-workflow truth, so later supervisor proof can widen the
  current preview-transform boundary instead of introducing a browser-local
  queue or host-local preview workflow shell

## g08.012 Batch 12.3 Outcome

- the existing `signal.runtime.preview-transform-boundary` now points at
  `docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md`
  and describes bounded preview-workflow queue and scheduling posture on the
  same runtime-owned seam as preview-transform and preview-device receipts
- the machine-readable supervisor boundary now closes the bounded preview-
  workflow consumer seam through the focused public runtime and stable
  host-edge proof spine instead of introducing a preview-queue-only
  acceptance lane
- `g08.012` is now complete, and the next `g08` queue is asset/session
  transform persistence, retention, and cache placement policy

## g08.013 Batch 13.1 Outcome

- `g08` now has a frozen transform-persistence contract in
  `docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md`
  instead of leaving asset/session transform persistence, retention, and cache
  placement as deferred prose under the earlier transform-artifact and
  preview-workflow seams
- asset/session transform persistence, retention, and cache placement are now
  required to compose through the closed media-service, transform-artifact,
  preview-transform, and preview-workflow seams rather than browser-local
  storage, editor-local session ledgers, or host-private cache policy
- runtime receipts, supervisor proof, and stable host-edge export remain
  explicitly deferred until later `g08.013` batches

## g08.013 Batch 13.2 Outcome

- `signal-runtime` now widens the existing transform-artifact seam with
  bounded asset/session transform persistence, retention, and cache-placement
  truth instead of opening a second cache-policy report family
- `RuntimeTransformArtifactSnapshot` now carries a typed
  `transform_persistence` summary covering persistence posture, retention
  policy class, retention authority and outcome, plus cache-placement posture,
  authority, and outcome
- public runtime and stable host-edge proofs now consume the same runtime-
  owned transform-persistence truth, so later supervisor proof can widen the
  current transform-artifact boundary instead of introducing a browser-local
  storage ledger or host-local cache-policy shell

## g08.013 Batch 13.3 Outcome

- `signal-supervisor-tools` now widens the existing shared
  `signal.runtime.transform-artifact-boundary` so it points at
  `docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md`
  and explicitly describes `transform_persistence` on the same runtime-owned
  seam as transform-artifact readiness, invalidation, and reuse
- the repo-owned proof path remains
  `effigy acceptance:transform-artifact-boundary`, so runtime, supervisor,
  clip-render, offline preview, and both stable host edges close the bounded
  persistence-policy seam without creating a second persistence-only
  acceptance shell
- `g08.013` is now complete, and the next `g08` queue is live external MIDI
  device ownership and backend parity depth

## g08.014 Batch 14.1 Outcome

- `g08` now has a frozen live external MIDI ownership and backend-parity
  contract in
  `docs/contracts/065-live-external-midi-device-ownership-and-backend-parity-contract.md`
  instead of leaving this seam implicit under the older external MIDI graph,
  controller-expression, live backend, and backend-parity contracts
- live external MIDI ownership, attach continuity, and backend parity are now
  required to compose through the closed external MIDI graph,
  controller-expression, live backend lifecycle, backend parity, and
  transform-persistence seams rather than backend-local endpoint policy or
  host-local device picks
- runtime receipts, supervisor proof, and stable host-edge export remain
  explicitly deferred until later `g08.014` batches

## g08.014 Batch 14.2 Outcome

- `signal-runtime` now widens the existing external MIDI seam with a typed
  `live_ownership` summary on `RuntimeExternalMidiEndpointGraphSnapshot`
  instead of opening a second live-MIDI-only report family
- the new runtime-owned receipt family carries ownership posture, attach
  continuity, backend parity, and guarded parity outcome, derived from the
  existing Linux-session and interruption seams rather than backend-local
  device picks or session-manager policy
- public runtime and stable host-edge proofs now consume the same runtime-
  owned live external MIDI truth, so later supervisor proof can widen the
  current external MIDI boundary instead of introducing a backend-local
  endpoint policy shell

## g08.014 Batch 14.3 Outcome

- `signal-supervisor-tools` now widens the existing shared
  `signal.runtime.external-midi-boundary` so it points at
  `docs/contracts/065-live-external-midi-device-ownership-and-backend-parity-contract.md`
  and explicitly describes `live_ownership` on the same runtime-owned seam as
  external MIDI discovery, graph, endpoint, capability, and route truth

## g08.016 Batch 16.3 Outcome

- `signal-supervisor-tools` now proves one grouped supervisor export can carry
  Linux live ownership, JACK coordination, PipeWire/ALSA parity, and
  clock-topology truth together instead of only listing isolated Linux boundary
  descriptors
- the repo-owned proof path remains
  `effigy acceptance:linux-live-acceptance-lane`, so the bounded Linux live
  acceptance seam closes through public runtime receipts, supervisor export,
  and both stable host edges without creating a daemon-local recovery shell
- `g08.016` is now complete, and the next `g08` queue is immersive render and
  monitoring acceptance depth

## g08.017 Batch 17.1 Outcome

- `g08` now has a frozen shared immersive render and monitoring acceptance
  contract in
  `docs/contracts/068-immersive-render-and-monitoring-acceptance-contract.md`
  instead of leaving grouped immersive proof fragmented across the existing
  spatial boundary and the earlier room-policy, deployment-monitoring, and
  renderer-export contracts
- the shared acceptance lane is now explicitly required to compose through
  public runtime receipts, supervisor export, and both stable host edges rather
  than renderer-private capability shells or product-local monitoring workflow
  glue
- grouped descriptor, Effigy lane, and broader advisory versus deferred
  immersive rerun depth remain intentionally deferred until later `g08.017`
  batches

## g08.017 Batch 17.2 Outcome

- `signal-supervisor-tools` now exposes one grouped
  `signal.runtime.immersive-acceptance-lane` descriptor so immersive
  room-policy, deployment-monitoring, and renderer-export acceptance can be
  inspected as one repo-owned seam instead of only through the broader spatial
  boundary
- the repo-owned proof path is now `effigy acceptance:immersive-acceptance-lane`,
  which composes the existing spatial boundary proof with the grouped
  descriptor instead of inventing a second renderer-private or workflow-local
  acceptance shell
- broader renderer-native reruns and richer monitoring-scene depth remain
  advisory or deferred until the grouped consumer proof lands in Batch 17.3

## g08.017 Batch 17.3 Outcome

- `signal-supervisor-tools` now proves one grouped supervisor export can carry
  immersive room-policy, deployment-monitoring, and renderer-export truth
  together instead of only listing the grouped immersive descriptor over the
  broader spatial seam
- the repo-owned proof path remains
  `effigy acceptance:immersive-acceptance-lane`, so the bounded immersive
  acceptance seam closes through public runtime receipts, supervisor export,
  and both stable host edges without creating a renderer-private or
  workflow-local acceptance shell
- `g08.017` is now complete, and the next `g08` queue is control-surface and
  preview workflow acceptance depth

## g08.015 Batch 15.1 Outcome

- `g08` now has a frozen shared cross-backend device protocol and live
  workflow acceptance contract in
  `docs/contracts/066-cross-backend-device-protocol-and-live-workflow-acceptance-contract.md`
  instead of leaving grouped device proof fragmented across the existing
  external MIDI, controller-expression, control-surface, advanced-hardware,
  and live ownership seams
- the later shared acceptance lane is now required to compose through public
  runtime receipts, supervisor export, and both stable host edges rather than
  backend-local endpoint policy or host-private workflow glue
- grouped descriptor, Effigy acceptance lane, and broader advisory or
  deferred device-depth policy remain explicitly deferred until later
  `g08.015` batches

## g08.015 Batch 15.2 Outcome

- `signal-supervisor-tools` now exposes one machine-readable
  `signal.runtime.device-workflow-acceptance-lane` descriptor instead of
  leaving grouped device-workflow proof spread across isolated external MIDI,
  controller-expression, control-surface, and advanced-hardware boundaries
- Effigy now owns one runnable
  `effigy acceptance:device-workflow-acceptance-lane` task that composes the
  already-closed bounded proof spine into one shared lane while keeping
  backend-native transport depth explicitly non-blocking
- `g08.015` now has a real grouped acceptance surface, and the remaining work
  is the final consumer-proof closure rather than more policy setup

## g08.015 Batch 15.3 Outcome

- one repo-owned supervisor export proof now demonstrates that external MIDI
  live ownership, controller-expression, control-surface posture, and
  advanced-hardware workflow receipts are consumable together instead of only
  through separate boundary-local descriptors
- `effigy acceptance:device-workflow-acceptance-lane` now composes the
  grouped descriptor, grouped export proof, and existing boundary proofs into
  one reusable shared acceptance lane
- `g08.015` is now complete, and the next `g08` queue is Linux live backend
  acceptance and failure-injection depth

## g08.016 Batch 16.1 Outcome

- `g08` now has a frozen shared live Linux backend acceptance contract in
  `docs/contracts/067-live-linux-backend-acceptance-and-failure-injection-contract.md`
  instead of leaving grouped Linux live-backend proof fragmented across the
  existing live ownership, JACK coordination, PipeWire/ALSA parity, and
  clock-topology seams
- the later shared acceptance lane is now required to compose through public
  runtime receipts, supervisor export, and both stable host edges rather than
  daemon-local policy or backend-specific recovery glue
- grouped descriptor, Effigy acceptance lane, and broader advisory or
  deferred Linux failure depth remain explicitly deferred until later
  `g08.016` batches

## g08.016 Batch 16.2 Outcome

- `signal-supervisor-tools` now exposes one machine-readable
  `signal.runtime.linux-live-acceptance-lane` descriptor instead of leaving
  grouped Linux live proof spread across isolated live ownership, JACK,
  PipeWire/ALSA, and clock-topology boundaries
- Effigy now owns one runnable `effigy acceptance:linux-live-acceptance-lane`
  task that composes the already-closed bounded proof spine into one shared
  lane while keeping backend-native daemon and recovery depth explicitly
  non-blocking
- `g08.016` now has a real grouped acceptance surface, and the remaining work
  is the final consumer-proof closure rather than more policy setup

## g08.017 Batch 17.3 Outcome

- one repo-owned supervisor export proof now demonstrates that immersive
  room-policy, deployment-monitoring, and renderer-export receipts are
  consumable together instead of only through the grouped descriptor and the
  broader spatial boundary task
- `effigy acceptance:immersive-acceptance-lane` now composes the grouped
  descriptor, grouped export proof, and existing spatial proof spine into one
  reusable shared acceptance lane
- `g08.017` is now complete, and the next `g08` queue is shared
  control-surface and preview workflow acceptance depth

## g08.018 Batch 18.1 Outcome

- `g08` now has a frozen shared control-surface and preview workflow
  acceptance contract in
  `docs/contracts/069-control-surface-and-preview-workflow-acceptance-contract.md`
  instead of leaving grouped workflow proof fragmented across the existing
  advanced-hardware and preview-transform seams
- the later shared acceptance lane is now required to compose through public
  runtime receipts, supervisor export, and both stable host edges rather than
  device-private page logic or browser-local queue policy
- grouped descriptor, Effigy acceptance lane, and broader device-native or
  browser-native workflow depth remain explicitly deferred until later
  `g08.018` batches

## g08.018 Batch 18.2 Outcome

- `signal-supervisor-tools` now exposes one machine-readable
  `signal.runtime.control-preview-workflow-acceptance-lane` descriptor instead
  of leaving grouped controller and preview workflow proof spread across the
  isolated advanced-hardware and preview-transform boundaries
- Effigy now owns one runnable
  `effigy acceptance:control-preview-workflow-acceptance-lane` task that
  composes the bounded workflow proof spine into one shared lane while keeping
  broader device-native and browser-native reruns explicitly non-blocking
- `g08.018` now has a real grouped acceptance surface, and the remaining work
  is the final consumer-proof closure rather than more policy setup

## g08.018 Batch 18.3 Outcome

- one repo-owned supervisor export proof now demonstrates that control-surface
  workflow, advanced-feedback, preview-device policy, and preview-workflow
  receipts are consumable together instead of only through the grouped
  descriptor and the isolated boundary tasks
- `effigy acceptance:control-preview-workflow-acceptance-lane` now composes
  the grouped descriptor, grouped export proof, and the existing advanced-
  hardware and preview-transform proof spine into one reusable shared
  acceptance lane
- `g08.018` is now complete, and the next `g08` queue is integrated
  live-ownership and workflow acceptance depth

## g08.019 Batch 19.1 Outcome

- `g08` now has a frozen shared integrated live-ownership and workflow
  acceptance contract in
  `docs/contracts/070-integrated-live-ownership-and-workflow-acceptance-contract.md`
  instead of leaving the broader closeout-facing proof split across four
  parallel grouped lanes only
- the later integrated lane is now required to compose through public runtime
  receipts, supervisor export, and both stable host edges rather than
  backend-local, device-private, browser-local, or renderer-private
  coordination glue
- integrated descriptor, Effigy acceptance lane, and broader repeated-run or
  closeout-adjacent depth remain explicitly deferred until later `g08.019`
  batches

## g08.019 Batch 19.2 Outcome

- `signal-supervisor-tools` now exposes one machine-readable
  `signal.runtime.integrated-live-ownership-and-workflow-acceptance-lane`
  descriptor that groups the closed Linux live, device workflow, immersive,
  and control-preview workflow acceptance seams instead of leaving integrated
  acceptance as four parallel grouped descriptors only
- Effigy now owns one runnable
  `effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane`
  task that composes those grouped lanes into one shared integrated seam
  while keeping repeated-run and environment-specific depth explicit and
  non-blocking
- the next remaining `g08.019` work is the grouped runtime, supervisor, and
  stable host-edge consumer proof closure rather than more descriptor setup

## g08.020 Batch 20.1 Outcome

- `g08` now has a frozen shared generation closeout and downstream workflow
  readiness contract in
  `docs/contracts/071-generation-closeout-and-downstream-workflow-readiness-gate-contract.md`
  instead of leaving the final `g08` verdict as an untyped review over the
  completed acceptance lanes only
- the later closeout gate is now required to compose through the closed
  `g08.019` integrated acceptance seam and one machine-readable closeout
  surface instead of product-local or CI-local closeout judgment
- concrete closeout descriptor, Effigy gate task, and final readiness verdict
  remain explicitly deferred to later `g08.020` batches

## g08.020 Batch 20.2 Outcome

- `signal-supervisor-tools` now emits one machine-readable `g08` closeout
  descriptor instead of leaving the final `g08` review as a docs-only or
  manual summary
- Effigy now owns one runnable `acceptance:g08-closeout` task that composes
  the closed `g08.019` integrated acceptance lane, the closeout descriptor
  proof, the descriptor export, and repo validation into one shared gate
- the final closeout verdict remains explicitly deferred to Batch 20.3, but
  the reusable `g08` closeout surface is now real rather than provisional

## g08.020 Batch 20.3 Outcome

- `signal-supervisor-tools` now records the final bounded `g08` closeout
  verdict, with each readiness area marked sufficient for closeout instead of
  review-only
- the shared closeout surface now points at
  `docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md`
  as the explicit post-`g08` queue instead of a self-referential placeholder
- `g08` is now complete, and broader repeated-run or environment-matrix depth
  is explicit backlog work rather than an implied still-active generation

## Next Task

COMPLETE. `g08` is closed. Promote
`docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md`
only when maintainers choose to open the post-`g08` generation.

## g08.019 Batch 19.3 Outcome

- one repo-owned supervisor export proof now demonstrates that Linux live
  ownership, device workflow, immersive render and monitoring, and
  control-preview workflow receipts are consumable together instead of only
  through the grouped descriptor and grouped Effigy lane
- `effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane`
  now composes the grouped export proof together with the four grouped lanes
  and the integrated descriptor, closing the shared integrated acceptance seam
- `g08.019` is now complete, and `g08.020` is the next queue for generation
  closeout and downstream workflow readiness

## g08.010 Batch 10.2 Outcome

- `signal-runtime` now widens the existing advanced-hardware seam with bounded
  scene-mapping, feedback-page, and safe action graph posture instead of
  opening a second controller-workflow report family
- `RuntimeAdvancedHardwareSnapshot` now carries aggregate scene-mapping,
  feedback-page, and safe-action-graph counts, while each advanced-hardware
  device descriptor exposes bounded authority and safe-action outcome answers
- public runtime and stable host-edge proofs now consume the same runtime-
  owned workflow truth, so later supervisor proof can widen the current
  advanced-hardware boundary instead of introducing a host-local workflow
  shell

## g08.010 Batch 10.3 Outcome

- the existing `signal.runtime.advanced-hardware-boundary` now points at
  `docs/contracts/061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md`
  and describes bounded scene-mapping, feedback-page, and safe action graph
  counts plus per-device posture, authority, and safe-action outcome on the
  same runtime-owned seam
- the machine-readable supervisor boundary now closes the bounded control-
  surface workflow consumer seam through the existing public runtime and stable
  host-edge proof spine instead of introducing a controller-workflow-only
  acceptance lane
- `g08.010` is now complete, and the next `g08` queue is preview-output
  routing, audition-sink ownership, and low-latency device policy
