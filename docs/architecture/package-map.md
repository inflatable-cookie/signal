# Package Map

Status: active
Owner: core-product
Updated: 2026-03-10
Vision refs: `docs/vision/001-signal-vision.md`
Architecture refs: `docs/architecture/system-architecture.md`

## Purpose

Freeze the first naming proposal for the extracted Signal workspace so research,
architecture, and implementation can converge on stable package and host names.
The Rust workspace now lives under `signal/crates/`, so package names remain
stable while their on-disk layout is grouped under one explicit workspace root.
The legacy C++ implementation now sits behind `signal/legacy/cpp/` as a
reference surface rather than sharing the active repo root with the Rust
workspace.

The main naming rule is:

- use `signal-<layer>` or `signal-<layer>-<domain>`
- prefer broad, reusable domain names over Finch-oriented feature names
- avoid vague buckets such as `signal-core` unless the content is truly
  irreducibly generic

## Naming Principles

1. `signal-primitives` is better than `signal-core`
   - `core` becomes a junk drawer
   - `primitives` makes the boundary explicit
2. `signal-analysis-rhythm` is better than `signal-beat`
   - the domain is larger than beat positions
   - onset, tempo, groove, meter, and confidence belong together
3. `signal-dsp-spectral` is better than `signal-spectral`
   - spectral work is a DSP layer, not a product-facing domain by itself
4. keep host-edge crates visibly separate from reusable DSP crates
   - plugin and hardware crates are integration boundaries, not algorithm homes

## Recommended Workspace Surface

### 1. Foundation

- `signal-primitives`
  - sample/frame/time/channel types
  - buffer primitives
  - realtime-safe utility types
- `signal-params`
  - parameter descriptors
  - smoothing/event primitives
  - modulation-facing parameter utilities
- `signal-midi`
  - MIDI event/model primitives
  - message normalization and routing helpers
- `signal-io`
  - audio decode/encode and probe surfaces
  - file/container helpers
  - shared offline asset loading

### 2. DSP

- `signal-dsp`
  - generic DSP kernels
  - filters, envelopes, dynamics helpers, metering primitives
- `signal-dsp-spectral`
  - FFT/STFT windows
  - spectral transforms and low-level spectral features
- `signal-dsp-resample`
  - sample-rate conversion
  - rate/timebase helpers

### 3. Analysis

- `signal-analysis`
  - shared analysis result types
  - confidence model
  - streaming/offline analysis traits
- `signal-analysis-rhythm`
  - onset detection
  - tempo estimation
  - beat tracking
  - meter/groove follow-ons
- `signal-analysis-tonal`
  - chroma extraction glue
  - key detection
  - tuning estimation
  - future chord/harmonic follow-ons
- `signal-analysis-loudness`
  - LUFS
  - true peak
  - loudness-range and related dynamics measurements
- `signal-analysis-character`
  - timbral descriptor extraction
  - energy and transient summary metrics
  - offline cataloging-oriented audio character summaries
- `signal-analysis-embed`
  - embedding inference
  - future classifier support

### 4. Execution

- `signal-graph`
  - graph model and execution semantics
  - explicit node/plan representation for executable graph structure, not only a flat stage chain
  - basic execution-class and latency metadata on nodes, so the plan can distinguish pure transforms, stateful processors, and latency-bearing processors
  - graph-owned planning-group summary for inline-realtime, stateful-realtime, and anticipative-eligible node groupings
  - graph-owned phased execution order derived from planning groups, so runtime can execute by plan phases instead of only a flat node list
  - graph-owned execution-lane order derived from planning phases, so anticipative and realtime work can be separated without host-local routing
  - graph-owned dispatch-plan construction derived from execution lanes, so runtime can execute a concrete anticipative/realtime dispatch sequence instead of only observing lane order
  - graph-owned anticipative prework handoff, so latency-bearing anticipative dispatches can prepare a reusable buffer before the realtime dispatch consumes it
  - routing, latency, tail, scheduling interfaces
  - executable stage-based graph processing for runtime-owned block work
- `signal-runtime`
  - embeddable engine/runtime orchestration
  - transport-facing runtime state
  - runtime-owned lifecycle control state, including handshake/configure/start history
  - runtime-owned degraded recovery stop/start visibility during sandbox restarts
  - typed recovery event emission for ordered restart tracing across soak paths
  - typed plugin-sandbox lifecycle milestone emission for ordered control-path tracing
  - runtime-owned sandbox control-plane tracing for ensure/handshake/load/create/prepare/activate
  - runtime-owned transport attach/detach tracing around sandbox lease and teardown edges
  - runtime-owned transport fault tracing for broker detach failures during recovery
  - runtime-owned heartbeat request/response/miss tracing across control-loop execution
  - runtime-owned block dispatch request/completion tracing across brokered render work
  - runtime-owned lease-rollover tracing when block sequencing crosses sandbox lease generations
  - runtime-owned completion-region and lease-epoch invalidation tracing during recovery boundaries
  - runtime-owned completion-slot transition tracing, including fallback application around failed render work
  - runtime-owned broker failure tracing around plan creation, payload I/O, and transport teardown
  - runtime-owned sandbox operation failure tracing derived from CLAP harness fault envelopes
  - runtime-owned block sequencing and continuity tracking across lease rollover
  - runtime-owned supervision, watchdog escalation, and readiness degradation
  - shared supervisor report types, including runtime-owned control, timeline, and automation snapshots
  - diagnostics and readiness surfaces
- `signal-supervisor-tools`
  - live runtime supervisor and soak-reporting CLI
  - real host scenario inspection outside the host `main` binaries
  - versioned text and machine-readable export for soak and restart scenarios
  - host-facing recovery execution detail, including last recovery intent and requested stop reason
  - runtime-owned `recovery_sequence` export for ordered degraded restart tracing
  - runtime-owned `lifecycle_sequence` export for ordered sandbox control-path tracing across startup, attach, teardown, and restart phases
  - runtime-owned `transport_sequence` and `heartbeat_sequence` export for broker churn and control-loop inspection
  - runtime-owned `block_dispatch_sequence` and `lease_rollover_sequence` export for render-work and lease-generation inspection
  - runtime-owned `invalidation_sequence` export for explicit recovery invalidation boundaries
  - runtime-owned `completion_slot_sequence` export for completion-state and fallback inspection
  - runtime-owned `transport_fault_sequence` export as the canonical top-level transport-fault view, with explicit source, phase, resource, and operation metadata for host-broker faults, sandbox-operation faults, and projected runtime-dispatch timeout/invalidation/fallback plus detach-lifecycle boundaries
  - runtime-owned `transport_fault_summary` export to freeze that top-level view as fault-adjacent only rather than a mirror of the full transport success path
  - runtime-owned `transport_session_summary` export for healthy-path transport/session visibility, including current attachment state, heartbeat freshness, dispatch state, concurrent active-session identity, per-session liveness (`heartbeat_freshness`, `dispatch_state`, `active_block_sequence`), and per-session transport-fault freshness/history inside `active_sessions`, without weakening the fault-adjacent transport fault boundary
  - runtime-owned transport-session admission policy for steady-state versus recovery-overlap attach intent, with local/server hosts consulting runtime admission state on broker attach/detach paths and during real overlapping recovery handoff
  - host recovery rollback for overlap-handoff failures, so replacement sessions are torn back down if old transport teardown or replacement startup fails after overlap admission
  - cleanup of rejected overlap prepares, so competing `RecoveryOverlap` attach attempts do not leave unadmitted broker regions or lifecycle state behind
  - staged recovery retry support while runtime is already stopped, so hosts can orchestrate interleaved teardown-fault and overlap-admission failures across more than one degraded recovery attempt
  - runtime-owned lingering transport-session state in `transport_concurrency_snapshot`, so `DetachRequested` and `DetachFaulted` sessions occupy admission capacity explicitly and survive as inspectable control state between failed recovery attempts
  - explicit lingering-session cleanup recovery path, so a later degraded recovery attempt can tear down a previously faulted origin session, free steady-state capacity, and restart from fresh transport admission without a full runtime reset
  - retryable lingering-session cleanup, so a faulted origin session may fail explicit cleanup once more and still recover on a later degraded attempt without breaking runtime admission state
  - orphan lingering-session sweep before fresh overlap attach, so failed replacement rollback sessions are cleaned out of runtime admission before the next recovery handoff instead of silently consuming overlap capacity
  - hard recovery abort when orphan cleanup lacks valid transport metadata, so hosts do not skip stale runtime-owned lingering state and proceed with an inconsistent broker boundary
  - multi-orphan lingering-session sweep for one sandbox, so recovery cleanup no longer assumes a single stale replacement-side transport candidate
  - post-start late lingering-session reconciliation, so a previously faulted origin teardown can finish after replacement attach without disturbing the active recovered session; late cleanup failures stay visible as lingering runtime state instead of forcing another stop
  - explicit strict-vs-best-effort lingering cleanup modes, so pre-attach cleanup can abort recovery while post-start reconciliation preserves the active replacement session and keeps stale lingering state visible
  - adjacent-episode lingering sweep before next overlap recovery, so a prior lingering origin can be cleaned before a newer lingering replacement recovers again, and stale metadata blocks that next recovery cleanly instead of over-admitting transport sessions
  - runtime-owned lingering-session provenance and cleanup ordering, so hosts consume runtime-planned cleanup candidates ordered by origin/replacement provenance, attach sequence, and attach epoch instead of maintaining that sorting policy locally
  - runtime-owned lingering cleanup scheduling state, so `active_sessions` show cleanup mode, attempt count, cleanup-in-progress, last cleanup epoch, and last cleanup error while hosts perform the actual broker teardown
  - runtime-owned lingering cleanup workflow APIs, so hosts execute typed `LingeringCleanupPlan` batches and report cleanup success/failure back into runtime instead of open-coding candidate loops over transport snapshots
  - runtime-owned pending lingering cleanup work and deferred retry scheduling, so hosts enqueue cleanup by trigger and drain runtime-issued work items while `transport_concurrency_snapshot.pending_cleanup_work_items` exposes queued cleanup pressure
  - runtime-owned lingering cleanup epochs and retry backoff metadata, so deferred cleanup work is only drainable when `ready_at_processing_epoch` has been reached and cleanup ordering stays inspectable via `cleanup_epoch`
  - runtime-owned sandbox-scoped lingering cleanup waves, so queued work, cleanup plans, and active sessions can distinguish one cleanup cycle from the next through `cleanup_wave`, while `transport_concurrency_snapshot.pending_cleanup_waves` exposes per-wave queue summaries
  - runtime-owned executable graph block processing and `engine_block_snapshot` metrics, so Signal can process a concrete graph buffer with stage-based nonlinear/stereo transforms and expose last-block execution state through the shared runtime report
  - runtime-owned graph execution context construction, so projection epoch, parameter epoch, configured block size, and transport state are attached to each processed block instead of being inferred from host-local execution loops
  - runtime-owned anticipative mode propagation into graph execution context, so the executable graph can phase its work according to current runtime planning state
  - runtime-owned mapping from graph projection nodes into executable plan structure, so both hosts drive the same node-shaped engine contract
  - runtime-owned planning-group refresh on graph apply and runtime reconfigure, so anticipative-enabled versus realtime-only execution produces different scheduler-facing graph summaries without host-local inference
  - runtime-owned aggregation of node execution classes, planning groups, phase order, execution lanes, dispatch sequence, and latency metadata into the shared engine snapshot, so scheduler-relevant plan hints are visible without host-local inference
  - runtime-owned reporting of anticipative prework versus realtime application, including prepared dispatch count, realtime dispatch count, handoff count, and last prework/realtime input peaks in `engine_block_snapshot`
  - runtime-owned anticipative prework cache with a short processing-epoch validity window keyed by graph/projection/parameter/input state, so adjacent matching blocks can reuse prepared work and report cache hits/misses through `engine_block_snapshot`
  - runtime-owned prework cache admission/consumption tracking, including queued next-block admissions and queued consumptions, so `engine_block_snapshot` distinguishes inline prepared work from ahead-of-time prepared work instead of flattening both into one prepared state
  - runtime-owned prework block-sequence freshness tracking, so prepared work expires on real block progression rather than only coarse processing-epoch changes
  - runtime-owned prework cache invalidation and retirement lifecycle, so `configure`, graph/transport updates, parameter batches, expiry, and input-signature churn retire stale prepared work and report cache state, invalidation reason, retirement reason, and whether retirement happened before consumption through `engine_block_snapshot`
  - runtime-owned future-state queued prework admission, so hosts can prime the next block against its expected parameter epoch and transport state and still consume that work when the matching control updates are later applied
  - runtime-owned transport-aware prework invalidation, so queued work survives explicit per-block transport progression when `playing`, `tempo_bpm`, and `timeline_position_samples` still match the admitted future state
  - runtime-owned bounded prework queue with explicit queue depth/capacity reporting and oldest-entry eviction when future anticipative admissions exceed runtime capacity
  - runtime-owned future-state replacement/reuse rules for queued prework, so identical target-state admissions reuse existing queued entries while changed future parameter or transport targets explicitly supersede the older queued work for that same block
  - runtime-owned prework planning-window reconciliation, so hosts can declare a future block horizon and runtime can proactively retire queued targets that fall out of that horizon instead of leaving queue revision as host-local bookkeeping
  - runtime-owned future block-sequence planning for the declared prework window, so hosts no longer carry a target-block deque and instead only build future-state targets for the block IDs runtime keeps in scope
  - runtime-owned prework stop/recovery invalidation, so queued anticipative work is retired when the runtime stops and does not leak across degraded restart boundaries
  - runtime-owned prework forecast profile derivation, expansion, requested/effective mode switching, recovery-safe persistence, and forecast-plan-change reconciliation, so hosts declare runtime role while runtime selects the default forecast profile, can switch between disabled/default/explicit/raw override modes, preserves explicit/raw intent across reconfigure and recovery, preserves compatible queued future prework while retiring only entries that fall out of the revised forecast plan, and derives the future transport projection, parameter batch, and primed input block for each planned target block
  - runtime-owned forecast-plan refill after reconciliation, so revised scheduler policy preserves compatible queued work, retires incompatible future entries, and automatically re-primes missing future targets inside the new planning window without waiting for later host progression
  - runtime-owned forecast-window lifecycle rebuild, so graph apply and runtime start/restart can proactively seed or restore the anticipative planning window from stored forecast state without requiring host-local boot priming
  - runtime-owned bounded prework runner, so the declared future window can exceed the currently prepared queue while runtime keeps pending future targets internally and drains them through an anticipative service cycle budget
  - runtime-owned explicit prework service lane, so future-window reconciliation and pending-target preparation are separate scheduler steps and hosts no longer rely on current-block forecast application to drain pending anticipative work
  - runtime-owned prework service-lane state machine, so the anticipative background lane exposes paused, pending, servicing, idle, disabled, and starved runtime states instead of behaving like an opaque repeated drain call
  - runtime-owned prework service pressure hints and adaptive service policy, so the background lane can throttle or yield under elevated or critical realtime pressure instead of treating every service cycle as an equal drain attempt
  - runtime-owned backlog-class prioritization for pending prework targets, so elevated pressure can keep servicing immediate and near-term future work while deferred targets stay queued instead of draining in plain block-sequence order
  - runtime-owned graph-semantic prework service policy, so elevated-pressure backlog servicing can widen for latency-focused anticipative graphs and tighten for plugin-constrained mixed graphs instead of applying one fixed backlog cutoff to every graph shape
  - runtime-owned sandbox-aware plugin gating for prework service, so active plugin sandbox count can fully yield the anticipative lane for plugin-constrained graphs under non-normal pressure and the shared engine snapshot can report both active sandbox count and whether that gate is currently active
  - runtime-owned plugin-backed node binding projection, so plugin-constrained scheduling can distinguish bound/active/degraded/missing sandbox ownership for plugin-backed realtime nodes instead of treating all plugin-heavy graphs as one undifferentiated bucket
  - host-assembly-driven plugin binding input, so local/server hosts now apply graph projection, plugin sandbox inventory, and plugin-backed node bindings from one declared assembly surface instead of separate demo helper paths
  - runtime-owned forecast-window sizing, maintenance, and policy persistence, so hosts no longer clamp a local horizon constant, pass a remaining-block cap, run a separate forecast-advance step, resend the same profile policy on every block, or keep a host-owned future-target deque while runtime reconciles how much of the future span to keep primed from current execution context plus stored forecast state
  - runtime-owned `broker_failure_sequence` export for concrete transport-side failure inspection
  - runtime-owned `sandbox_operation_failure_sequence` export for sandbox-emitted attach/flush/protocol failure inspection
  - contract-bound export envelope for automation and external tooling
- `signal-ipc`
  - runtime control protocol
  - message/event model shared across hosts and consumers

### 5. Trust-Edge Integration

- `signal-plugin`
  - plugin-host abstractions
  - common instance/state/parameter surfaces
- `signal-plugin-clap`
  - CLAP adapter
  - typed CLAP sandbox failure classification for attach/flush/protocol fault stages
- `signal-plugin-vst3`
  - VST3 adapter
- `signal-plugin-au`
  - AU adapter
- `signal-plugin-lv2`
  - LV2 adapter
- `signal-hardware`
  - common audio/MIDI device abstractions
  - device model and diagnostics contracts
- `signal-hardware-coreaudio`
  - macOS backend
- `signal-hardware-wasapi`
  - Windows shared/exclusive backend

Linux backends can be added later only when real implementation pressure
appears:

- `signal-hardware-alsa`
- `signal-hardware-jack`

### 6. Host Assemblies

- `signal-host-local`
  - local desktop runtime host
  - current end-to-end proof host for runtime-owned graph block execution plus sandbox control/recovery
- `signal-host-server`
  - headless/remote runtime host
  - mirrors the runtime-owned graph block execution path in a server profile, so both host assemblies exercise real runtime work instead of only transport/supervision control flow
- `signal-plugin-sandbox`
  - out-of-process plugin container

## What I Would Not Freeze As Long-Term Names

- `signal-core`
  - too vague; likely to become a dumping ground
- `signal-beat`
  - too narrow and too Finch-shaped
- `signal-tonal`
  - workable, but weaker than the consistent `signal-analysis-*` family
- `signal-loudness`
  - same issue; better grouped under analysis
- `signal-spectral`
  - spectral transforms belong in the DSP layer

## First Concrete Freeze

If we want the smallest useful first batch, I would freeze these names first:

- `signal-primitives`
- `signal-io`
- `signal-dsp`
- `signal-dsp-spectral`
- `signal-analysis`
- `signal-analysis-rhythm`
- `signal-analysis-tonal`
- `signal-analysis-loudness`
- `signal-graph`
- `signal-runtime`
- `signal-plugin`
- `signal-plugin-clap`
- `signal-hardware`
- `signal-host-local`
- `signal-host-server`
- `signal-plugin-sandbox`

Then add these only when implementation pressure justifies them:

- `signal-params`
- `signal-midi`
- `signal-dsp-resample`
- `signal-analysis-embed`
- `signal-plugin-vst3`
- `signal-plugin-au`
- `signal-plugin-lv2`
- `signal-hardware-coreaudio`
- `signal-hardware-wasapi`

## Current Workspace State

These packages now exist as real workspace members under `crates/`:

- `signal-primitives`
- `signal-dsp`
- `signal-dsp-spectral`
- `signal-analysis`
- `signal-analysis-rhythm`
- `signal-analysis-tonal`
- `signal-analysis-loudness`
- `signal-graph`
- `signal-runtime`
- `signal-ipc`
- `signal-plugin`
- `signal-plugin-clap`
- `signal-plugin-sandbox`
- `signal-hardware`
- `signal-hardware-coreaudio`
- `signal-host-local`
- `signal-host-server`

These names should be treated as frozen implementation targets unless a later
architecture batch explicitly changes them.

## Layout Note

The Rust workspace packages now live under:

```text
signal/
  crates/
    signal-primitives/
    signal-dsp/
    signal-dsp-spectral/
    signal-analysis/
    signal-analysis-rhythm/
    signal-analysis-tonal/
    signal-analysis-loudness/
    signal-graph/
    signal-runtime/
    signal-ipc/
    signal-plugin/
    signal-plugin-clap/
    signal-plugin-sandbox/
    signal-hardware/
    signal-hardware-coreaudio/
    signal-host-local/
    signal-host-server/
    signal-supervisor-tools/
```

This keeps the repository root reserved for repo-level concerns such as the
legacy C++ implementation, docs, top-level build surfaces, and workspace
manifests.

## Next Task

Decide whether the payload-only debug policy is now sufficiently frozen to
leave this export boundary alone for a while, or whether there is a concrete
inspection need strong enough to justify a second explicit debug section.
