<pre>
 ░▒▓███████▓▒░▒▓█▓▒░░▒▓██████▓▒░░▒▓███████▓▒░ ░▒▓██████▓▒░░▒▓█▓▒░
░▒▓█▓▒░      ░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░
░▒▓█▓▒░      ░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░
 ░▒▓██████▓▒░░▒▓█▓▒░▒▓█▓▒▒▓███▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓████████▓▒░▒▓█▓▒░
       ░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░
       ░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░
░▒▓███████▓▒░░▒▓█▓▒░░▒▓██████▓▒░░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓████████▓▒░

  L O O P H O L E - S I G N A L
</pre>

# Shared audio-systems runtime and DSP workspace

Signal is the shared audio-systems repo for Loophole, Finch, and future apps.
Its active surface is the Rust library/runtime workspace under `crates/`.
The legacy C++ engine/runtime implementation now lives behind a reference
boundary under `legacy/cpp/`.

Signal may run out-of-process where isolation is the right trade, but the repo
itself is no longer defined by one mandatory standalone process topology.

## Responsibilities

Signal is responsible for:

- Real-time audio processing
- MIDI input/output handling
- Runtime plugin backend integration (VST3/CLAP backends in-tree)
- Processing graph execution
- Sample-accurate timing-sensitive engine behavior
- Engine telemetry and diagnostics emission

Signal is not responsible for project editing/state ownership (Pulse) or UI
behavior (Aura/Spark/Finch UI).

## Current Repository Layout

```
crates/
  signal-primitives/        # Shared sample/frame/buffer/time primitives
  signal-dsp/               # General reusable DSP kernels
  signal-dsp-spectral/      # FFT/STFT and spectral transforms
  signal-analysis/          # Shared analysis traits and result types
  signal-analysis-rhythm/   # Onset, tempo, beat, meter
  signal-analysis-tonal/    # Chroma, tuning, key, harmonic follow-ons
  signal-analysis-loudness/ # LUFS, true peak, LRA
  signal-graph/             # Graph model and execution semantics
  signal-runtime/           # Embeddable runtime orchestration
  signal-ipc/               # Shared runtime control/message seam
  signal-plugin/            # Format-neutral plugin abstractions
  signal-plugin-clap/       # CLAP adapter shell
  signal-plugin-sandbox/    # Out-of-process plugin container shell
  signal-hardware/          # Common device abstractions
  signal-hardware-coreaudio/# CoreAudio backend shell
  signal-host-local/        # Local desktop runtime host shell
  signal-host-server/       # Headless runtime host shell
  signal-supervisor-tools/  # Live supervisor and soak-reporting CLI
docs/               # Local docs/spec notes
legacy/
  README.md         # Legacy/reference boundary notes
  cpp/
    src/            # Legacy C++ engine/runtime source tree
    tests/          # Legacy C++ engine tests
    CMakeLists.txt  # Legacy C++ build surface
CMakeLists.txt      # Root compatibility wrapper for legacy/cpp
Cargo.toml
```

Northstar-aligned planning and research docs now live under `docs/`.

## Active vs Legacy

Active implementation direction:

- Rust crates under `crates/`
- Northstar-shaped docs under `docs/`

Reference-only implementation surface:

- legacy C++ runtime under `legacy/cpp/`

That C++ tree still builds and remains useful for migration/reference work, but
it is no longer the primary repo surface.

## Development

Use Effigy as the default command surface inside `signal/`:

```bash
effigy tasks
effigy doctor
effigy health
effigy dev
effigy validate
effigy qa:docs
```

Equivalent raw CMake flow:

```bash
cmake -S legacy/cpp -B legacy/cpp/build
cmake --build legacy/cpp/build --config Debug
ctest --test-dir legacy/cpp/build --output-on-failure
```

The root CMake entrypoint now wraps `legacy/cpp/`. New Rust work should prefer
the Cargo workspace and repo-owned Effigy tasks.

Rust workspace bootstrap:

```bash
cargo check --workspace
cargo run -p signal-host-local
cargo run -p signal-supervisor-tools -- --describe-export --format=json
cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json
cargo run -p signal-supervisor-tools -- --describe-release-boundary --format=json
cargo run -p signal-supervisor-tools -- --describe-generation-closeout --format=json
cargo run -p signal-supervisor-tools -- --format=json local soak
cargo run -p signal-supervisor-tools -- --format=json --include-payload local soak
```

The supervisor tool now emits a versioned JSON export schema with both
host-derived execution/transport/fault summaries and the shared runtime
supervisor report. Payload detail is excluded by default and can be added
explicitly with `--include-payload` for debugging and soak inspection. The
`host_summary` export also declares its included section list so automation can
distinguish default exports from payload-augmented debug runs without relying
on implicit shape assumptions, and it now also declares both supported and
enabled debug sections so the current payload-only debug policy is explicit in
the export itself. Use `--describe-export` when tooling needs that frozen
export policy without booting a host scenario. Use
`--describe-conformance-matrix` when tooling needs the runnable shared boundary
proof set, and `--describe-release-boundary` when tooling needs the first
packaging/versioning baseline for that same boundary without booting a host
scenario. Use `--describe-generation-closeout` when tooling needs the combined
`g04` closeout record, including residual risk and the explicit post-`g04`
queue handoff. The shared supervisor report now
also carries runtime control-path state, including handshake/configure/start
history and the current running/configured status. Host watchdog recovery now
also drives runtime `stop(DegradedModeRecovery)` / `start()` cycles, so
degraded sandbox restart episodes appear in the shared control snapshot rather
than only in host-local behavior. The host-facing execution summaries and
`signal-supervisor-tools` export also carry the last host recovery intent and
requested stop reason so restart handling remains inspectable without scraping
runtime event streams. The shared supervisor report JSON now also emits an
ordered `recovery_sequence`, so soak tooling can inspect degraded restart
episodes as a typed event trail instead of inferring them only from final
state. It also emits a typed `lifecycle_sequence` for the exercised
plugin-sandbox control milestones, including sandbox ensure/handshake,
plugin-type load, instance create/prepare/activate, transport attach, and the
deactivate/reset/destroy/transport-teardown/restart phases that bracket
recovery. Those lifecycle milestones are emitted from host control requests
rather than inferred from sandbox responses, so the ordered sequence tracks
the real control path across both steady startup and degraded restart handling.
The shared supervisor report now also emits a typed `transport_sequence` and
`heartbeat_sequence`, so soak tooling can inspect broker attach/detach churn,
detach faults, and heartbeat request/response/miss markers directly instead of
reconstructing them from host counters. It now also emits a typed
`block_dispatch_sequence` and `lease_rollover_sequence`, so restart and soak
analysis can correlate broker churn with actual render dispatch/completion work
and lease generation changes across epochs. It now also emits a typed
`invalidation_sequence`, so recovery analysis can see when completion regions
and lease epochs were explicitly invalidated before teardown instead of
inferring that only from later detach or restart activity.
It now also emits a typed `completion_slot_sequence`, so soak tooling can see
the exact completion-slot path around brokered render work: ready-for-
processing, processing, completed, timed-out, invalidated, and explicit
fallback-applied milestones.
It now also emits a typed `broker_failure_sequence`, so transport-side failure
points such as prepare-plan creation, payload write/read, broker destroy, and
transport teardown are visible in the shared supervisor report instead of
appearing only as generic resource errors.
It now also emits a typed `sandbox_operation_failure_sequence`, so sandbox-
emitted attach, flush, and protocol-failure paths from the CLAP harness are
distinguished from host-visible broker I/O failures in the shared report.
It now also emits a typed `transport_fault_sequence`, which is the canonical
top-level transport-fault view for supervision and tooling. That aggregate
sequence keeps explicit `source` labels for host-broker versus sandbox-
operation faults, plus runtime-dispatch projections for explicit invalidation,
fallback application, timed-out completion slots, and detach lifecycle
boundaries. It also carries typed `phase`, `resource`, and concrete
`operation` metadata so prepare, dispatch, teardown, and control-path failures
remain inspectable without dropping immediately to the subordinate sequences.
The broker-specific and sandbox-specific sequences stay available as
subordinate detail views. The shared supervisor report also emits a
`transport_fault_summary` that freezes this top-level boundary as
fault-adjacent only, so tooling can inspect source/phase counts without
mistaking the top-level view for a generic mirror of the full healthy
transport path. A separate `transport_session_summary` now provides that
healthy-path transport/session visibility explicitly, including attach/detach,
heartbeat, and block-dispatch counts plus current attachment state, heartbeat
freshness, dispatch state, current attached-session counts, and active or last
sandbox/lease/region identity. Concurrent `active_sessions` now also carry
their own per-session heartbeat freshness, dispatch state, and last active
block sequence. They now also carry per-session transport-fault history from
the canonical runtime transport-fault stream, so tooling can inspect mixed
concurrent-session liveness and fault freshness without collapsing back to the
top-level session fields.
`transport_session_summary` should now be treated as stable for schema version
1. Separately, `signal-runtime` now owns a transport-session admission policy
for steady-state versus recovery-overlap attach intent, and the local/server
hosts consult that policy on real broker attach/detach paths instead of
relying only on post-facto observation. Recovery now exercises a real
overlapping broker-session handoff: the replacement sandbox lifecycle attaches
its new broker transport before the old transport is torn down, so runtime
concurrency and lease-handoff behavior are no longer only inferred from
separate recovery steps. That overlap path now also rolls the replacement
session back if old transport teardown fails after replacement attach or if
replacement startup fails before runtime returns to `Ready`, so failed handoffs
do not leak attached broker sessions. Runtime admission now also rejects a
second concurrent `RecoveryOverlap` attach until the prior overlap session is
fully torn down, and rejected overlap prepares clean their lifecycle and broker
transport back up instead of leaving half-admitted state behind. Recovery can
also now retry while runtime is already stopped from a previous degraded
attempt, which lets teardown-fault and overlap-admission failures interleave
across more than one recovery episode without wedging control flow. Runtime
transport concurrency now also treats `DetachRequested` and `DetachFaulted`
sessions as first-class lingering state instead of leaving detach latency only
in host-local failure branches: `transport_concurrency_snapshot` exposes
lingering counts plus per-session state, and dedicated deferred-teardown
recovery tests prove a failed old-session teardown can leave one runtime-owned
lingering session in place between recovery attempts. The next recovery attempt
can now explicitly clean that lingering session up and resume from a fresh
runtime-owned transport admission path without forcing a full runtime reset,
which makes detach-latency recovery actionable instead of merely observable.
That cleanup path can also fail once more and later succeed on a subsequent
degraded recovery attempt, so repeated detach churn is now part of the tested
engine control flow rather than an unmodeled edge case. Hosts now also sweep
orphan lingering sessions for the same sandbox before opening a fresh overlap
attach, so failed replacement rollback state does not silently consume
recovery-overlap capacity on the next attempt. If that orphan cleanup cannot
reconstruct a valid transport boundary from runtime-owned metadata, recovery
aborts instead of skipping stale lingering state. That orphan sweep now covers
more than one lingering session candidate for the same sandbox, so recovery no
longer assumes there is at most one stale replacement-side transport to clean
up before a fresh handoff. Once a replacement session is already active, hosts
now also reconcile late lingering origin teardown completion as a post-start
cleanup step: successful reconciliation frees the lingering slot without
disturbing the active replacement session, while a failed late cleanup leaves
the lingering session visible and records the teardown fault instead of taking
the recovered runtime back down. Lingering cleanup is now also split explicitly
by control intent: strict pre-attach cleanup aborts recovery if stale lingering
transport cannot be reconstructed cleanly, while best-effort post-start
reconciliation may leave the lingering session visible and keep the active
replacement session running. Adjacent recovery episodes now use that split
directly, so an older lingering session can be swept before the next overlap
recovery from a newer lingering replacement, or block that next recovery
cleanly if its metadata is still invalid.
Lingering cleanup provenance and candidate ordering now also come from
`signal-runtime`: active lingering sessions carry `provenance`,
`attach_sequence`, and `attach_processing_epoch`, and hosts consume a
runtime-ordered cleanup candidate list instead of sorting origin/replacement
lingerers locally.
Runtime now also owns lingering cleanup scheduling state for those sessions:
active lingering entries track cleanup attempt count, last cleanup mode,
cleanup-in-progress state, last cleanup epoch, and the last cleanup error, so
strict pre-attach cleanup and best-effort post-start reconciliation are visible
as runtime transport state instead of only host-local control flow.
That cleanup state now has an explicit runtime workflow surface as well:
hosts request a typed `LingeringCleanupPlan` from runtime, execute its
candidates against the broker boundary, and report success or failure back
through runtime-owned completion APIs instead of open-coding cleanup loops over
runtime snapshots.
Runtime now also owns pending lingering cleanup work and deferred retry
issuance: hosts enqueue cleanup work by trigger (`RecoveryPreAttach` or
`PostStartReconciliation`), drain runtime-issued work items, and best-effort
cleanup failure schedules a runtime-owned `DeferredRetry` work item instead of
leaving retry timing entirely to host-local policy. That state is visible in
`transport_concurrency_snapshot.pending_cleanup_work_items`.
That work queue now has real scheduler state instead of immediate retry-only
behavior: cleanup work carries a runtime-owned `cleanup_epoch` plus
`ready_at_processing_epoch`, and deferred retries back off by processing epoch
instead of being immediately drainable in the same pass. The shared transport
concurrency snapshot now exposes deferred-retry count, next cleanup epoch, and
the oldest pending ready epoch.
Cleanup scheduling is now also grouped into sandbox-scoped cleanup waves:
queued work carries a `cleanup_wave`, retries stay inside the same wave, and
`transport_concurrency_snapshot.pending_cleanup_waves` exposes one summary per
sandbox/wave so late-detach reconciliation cycles can be distinguished from the
next cleanup pass instead of only being inferred from global cleanup epochs.
`signal-graph` and `signal-runtime` now also have a real executable block path:
Signal can apply a stage-based graph projection, process a concrete audio
buffer through it, and expose runtime-owned last-block metrics through
`engine_block_snapshot`. Both the local and server hosts now exercise that
graph block path in their block loops alongside the existing sandbox proof, so
the engine boundary is no longer only control/recovery scaffolding.
That execution path now also carries a runtime-owned graph execution context:
projection epoch, parameter epoch, configured block size, and transport state
travel with each processed engine block, and runtime advances transport
position after successful graph execution instead of leaving that context
implicit in host code. The graph projection itself now also has a node/plan
shape rather than only one flat stage list, so the runtime boundary is
starting to resemble scheduler-facing engine structure instead of only
per-block transforms.
That node/plan shape now also carries basic execution semantics: nodes declare
whether they are pure transforms, stateful processors, or latency-bearing
processors, and the shared runtime snapshot exposes node counts plus aggregate
latency metadata so the execution plan says something useful about scheduling.
That planning surface is now active in runtime rather than dormant graph
metadata: `engine_block_snapshot` carries runtime-owned planning groups for the
active graph plan, including inline-realtime, stateful-realtime, and
anticipative-eligible node counts plus the per-node planned view. The local
host currently runs that plan with anticipative execution enabled, while the
server host proves the same latency-bearing node shape can be replanned into
realtime execution when anticipative mode is off.
Runtime now also executes the graph by those planning phases instead of a flat
node list: the active anticipative mode travels in the execution context, the
graph processor derives a phase order from the current planning groups, and the
shared engine snapshot exposes phase count, anticipative-phase count, and phase
order for the last executed plan.
Runtime now also executes an explicit dispatch sequence derived from those
lanes, so the shared engine snapshot exposes dispatch count, dispatch
boundaries, and dispatch order rather than only phase and lane metadata. The
current model is still single-threaded, but the anticipative/realtime split is
now represented as a concrete runtime dispatch policy instead of only
classification.
That dispatch path now has a real handoff shape inside the engine: when
anticipative work is enabled, `signal-graph` produces a prepared prework
buffer from the anticipative dispatch before the realtime dispatch applies the
remaining work. The shared engine snapshot now exposes prepared-dispatch
counts, realtime-dispatch counts, dispatch handoff counts, and the last
prework/realtime input peaks, so the runtime can prove a genuine
background-style preparation boundary without introducing threads yet.
That handoff is now also reusable across adjacent blocks inside
`signal-runtime`: a small runtime-owned prework cache keeps prepared
anticipative work alive for a short processing-epoch validity window when the
graph, projection, parameter epoch, and input signature still match. The
shared engine snapshot now exposes cache hits, misses, last-hit status,
valid-until epoch, and last prework source epoch.
That cache now also has explicit admission and consumption reporting plus a
block-sequence freshness surface, so the engine snapshot distinguishes when
runtime admitted reusable prework, when later blocks actually consumed that
prepared work, and how many future block steps of reuse remain before the
prepared result expires.
It now also supports explicit queued admission for a target future block, so
hosts can prime the next block between iterations and the engine snapshot can
distinguish inline prework from ahead-of-time queued prework that was later
consumed or retired before use.
That queued admission path can now also target the future parameter epoch and
transport state expected for the next block, so matching next-block parameter
or transport updates no longer force queued prework to retire before it can be
consumed.
Hosts now also apply deterministic per-block transport projections before
engine execution and queue the next block against that same future transport
state, so queued prework can survive explicit transport/timeline progression
instead of only implicit runtime transport advancement. Runtime transport
invalidation now compares the same scheduler-facing transport fields as the
prework matcher (`playing`, `tempo_bpm`, and `timeline_position_samples`)
rather than invalidating on unrelated loop metadata differences.
That prework path is now backed by a small bounded runtime-owned queue instead
of a single cache slot. The shared engine snapshot now exposes queue capacity,
current queue depth, and peak queue depth, and runtime will explicitly evict
the oldest future entry when new queued work exceeds capacity rather than
silently overwriting or flattening future admissions.
That queued path now also treats future state as the replacement boundary:
re-admitting the same future block with the same parameter/transport/input
target reuses the existing queued entry instead of churning the queue, while a
changed future parameter or transport target for that same block explicitly
retires the older queued entry with `SupersededByAdmission`.
The local and server host flows now keep a small multi-block horizon primed
instead of only one next block, so the bounded queue is doing real scheduler
work rather than acting as a single-slot compatibility layer.
That horizon is now declared to runtime as an explicit planning window rather
than host-local repeated single-block admissions. Runtime can retire queued
future work that falls out of the declared window with
`PlanningWindowRevised`, while preserving matching future targets that remain
inside the window.
Runtime now also owns the future block-sequence planning for that window, so
hosts no longer carry a target-block deque just to decide which future block
IDs should be primed next. When anticipative prework is disabled, the runtime
planner now returns an empty window instead of burning future block IDs, and a
runtime stop/recovery cycle retires any queued prework before restart so stale
future work is not carried across the control boundary.
Runtime now also owns the deterministic future-state forecast for that window.
Hosts only declare runtime role and current execution context, while runtime
stores the active forecast profile or raw-policy override, derives the future
transport projection, parameter batch, and primed input block for each planned
target block, and can now move between `Disabled`, role-default,
explicit-profile, and raw-policy forecast modes as first-class runtime state.
It now also keeps requested mode separate from effective mode, so explicit/raw
override intent can survive reconfigure and recovery while the effective mode
still drops to `Disabled` whenever anticipative planning is off. That
runtime-owned forecast mode also owns the prework window size itself, and it
lives in `signal-runtime` instead of being passed on every block. Hosts no
longer choose the anticipative horizon depth directly, pass a remaining-block
cap, run a separate forecast-advance step, or select local vs server forecast
policy in their boot path. They apply the current block's forecast execution
context, and runtime reconciles the future prework window automatically from
stored forecast state within the declared window. Runtime now also treats
forecast plan changes as scheduler invalidation boundaries, so queued future
prework is explicitly retired when forecast profile/policy/mode changes alter
the future plan instead of silently surviving into the wrong scheduler state.
It now preserves queued future work that still matches the revised forecast
plan, trims only the entries that actually fall out of scope, and
automatically re-primes any missing future targets inside the revised window
instead of waiting for later host-driven block progression to fill those
holes.
That scheduler ownership now also extends into the runtime lifecycle itself:
applying a graph projection or starting/restarting the runtime can proactively
rebuild the current forecast window from stored runtime forecast state, so
hosts no longer need a dedicated boot-time `apply_forecast_state_for_block(0,
0)` call just to seed anticipative work.
The anticipative window is now also a real bounded work cycle rather than an
all-at-once prepare step. Runtime can keep a larger planned future window than
the currently prepared queue, carry pending future targets internally, and
prepare only a budgeted subset of that window per service cycle.
That bounded work no longer advances only as a side effect of current-block
forecast application: runtime now exposes an explicit prework service lane, so
hosts can advance pending future targets independently of the realtime block
path while forecast application itself only reconciles the declared future
window.
That lane now also has explicit runtime scheduler states: `Disabled`, `Idle`,
`Pending`, `Servicing`, `Paused`, and `Starved`, plus pause/resume/starvation
counters in the shared engine snapshot.
It now also has an explicit pressure model. Hosts can mark the lane
`Normal`, `Elevated`, or `Critical`, and runtime will throttle or yield the
background prework lane accordingly instead of draining every service cycle
the same way under timeout or watchdog pressure.
That pressure path now also has explicit backlog prioritization. Runtime
classifies pending future targets as `Immediate`, `NearTerm`, or `Deferred`,
surfaces those counts in the shared engine snapshot, and under elevated
pressure it will keep servicing only the nearer backlog classes while leaving
deferred future work queued instead of draining the full pending window in
plain block-sequence order.
That backlog policy is now also graph-semantic-aware. `signal-runtime`
derives a prework service semantic policy from the current graph planning
shape, and latency-focused anticipative graphs can keep a wider elevated-
pressure service scope than the current balanced demo graphs instead of using
one fixed backlog rule for every graph.
Mixed graphs that keep plugin-backed nodes on the realtime side now tighten
that same policy into `PluginConstrained`, so elevated pressure only services
immediate future work instead of widening background scope just because the
graph also contains latency-bearing anticipative nodes.
That plugin-constrained path is now sandbox-aware too. Runtime carries active
plugin sandbox count into the same prework service policy, exposes
`prework_service_active_plugin_sandboxes` plus
`prework_service_plugin_gate_active` in the shared engine snapshot/export
surface, and can fully yield the background prework lane under elevated or
critical pressure when more than one active plugin sandbox is competing with
plugin-backed realtime work.
That path is now also bound to actual plugin-backed node ownership instead of
only a coarse sandbox count. Hosts can project plugin-backed node bindings
into runtime, the shared engine snapshot/export now reports bound/active/
degraded/missing plugin-backed sandbox counts, and plugin-constrained gating
can react to the live transport-session state of the specific sandbox that a
plugin-backed realtime node belongs to.
Those bindings now come from the same host assembly input that defines the
graph projection and required plugin sandboxes, so runtime scheduling is fed
from one assembly seam instead of a separate demo-only binding helper.
It now also has explicit retirement tracking, so the engine snapshot can
distinguish prework that was retired before any future consumption from
prework that had already been consumed and was only being cleaned up after
churn invalidated it.
That cache now has explicit runtime lifecycle and invalidation semantics
rather than only implicit reuse. `configure`, graph projection changes,
transport projection changes, non-empty parameter batches, expired validity,
and input-signature mismatches can all retire cached prework before the next
block. The shared engine snapshot now exposes cache state, invalidation count,
retirement count, last invalidation reason, last retirement reason, and
whether the last retirement happened before consumption.
That phased path now also has explicit execution lanes: anticipative work and
realtime work are separated in the executable graph, and the shared engine
snapshot exposes lane count, anticipative-lane count, and lane order for the
active execution policy.
That sandbox-operation classification now lives in `signal-plugin-clap`
instead of being re-derived separately in each host assembly, so the CLAP
adapter owns the meaning of those failure stages.

All Rust workspace packages now live under `crates/`. Keep new Rust packages
under that directory rather than adding more top-level package folders.

Current trust-edge workspace shells:

- `signal-ipc`
- `signal-plugin`
- `signal-plugin-clap`
- `signal-plugin-sandbox`
- `signal-hardware`
- `signal-hardware-coreaudio`
- `signal-host-local`
- `signal-host-server`
- `signal-supervisor-tools`

## Documentation

Use the local docs bundle for architecture, research, roadmaps, and logs:

```bash
open docs/README.md
```

Key entry points:

- `docs/vision/001-signal-vision.md`
- `docs/architecture/system-architecture.md`
- `docs/contracts/001-shared-dsp-and-host-boundary.md`
- `docs/research/master-index.md`

## Real-Time Safety

Real-time code paths must avoid allocation, blocking calls, lock contention, and unbounded work. Treat plugin code as untrusted and keep API boundaries defensive.

## Licence

Signal is provided under the MIT Licence with the following additional clause:

**The Loophole name (including its components: Signal, Pulse, Aura and Chorus)
may not be used to promote or endorse any derived product without prior written
permission from the copyright holder.**

This clause applies to all repositories within the Loophole ecosystem.
