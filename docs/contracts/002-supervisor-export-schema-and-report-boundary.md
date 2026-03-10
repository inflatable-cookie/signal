# 002 Supervisor Export Schema And Report Boundary

Status: active
Owner: core-product
Updated: 2026-03-10
Related architecture: `docs/architecture/system-architecture.md`
Related package map: `docs/architecture/package-map.md`

## Purpose

Freeze the boundary between the shared runtime supervisor report surface and
the versioned CLI export produced by `signal-supervisor-tools`, so automation
and host tooling consume a stable contract instead of scraping ad hoc output.

## Contract

1. The machine-readable supervisor export envelope uses:
   - `schema = "signal.supervisor.export"`
   - `schema_version = 1`
2. The export envelope contains exactly four top-level payload areas:
   - `profile`
   - `scenario`
   - `host_summary`
   - `supervisor_report`
3. `supervisor_report` is the shared authority surface for runtime-level
   diagnostics, supervision state, timeline continuity, and any continuity data
   promoted into `signal-runtime`.
   In schema version 1, that also includes runtime-owned executable
   graph/block metrics through `engine_block_snapshot`, regardless of whether
   the current host is local or server.
   That engine snapshot now also carries the runtime-owned execution context
   for the last processed block, including projection/parameter epochs and
   transport state, so scheduler-facing engine inputs are visible without being
   duplicated into host-local summary fields. In the current runtime shape,
   that snapshot also reflects a node/plan-based graph projection rather than
   only a flat stage chain, including basic execution-class and latency
   metadata for the active graph plan. It also includes the runtime-owned
   planning-group view derived from those hints under the active
   anticipative/realtime configuration, so schema version 1 exposes both raw
   node classes and the current execution grouping for the active plan. That
   engine snapshot also carries the runtime-owned phase order derived from that
   grouping, so the shared report can describe how the current plan is
   actually executed rather than only how it is classified. It also carries the
   current execution-lane order, so the shared export can distinguish
   anticipative work from realtime work at the runtime engine boundary. It now
   also carries the runtime dispatch sequence derived from those lanes,
   including dispatch count, dispatch boundaries, and dispatch order, so the
   shared report can describe the active block-execution policy instead of only
   the plan structure. That same engine snapshot now also carries the
   anticipative prework handoff surface: prepared dispatch count, realtime
   dispatch count, dispatch handoff count, `last_prework_output_peak`, and
   `last_realtime_input_peak`. It also now carries the runtime-owned prework
   cache surface: `prework_cache_enabled`, `prework_cache_admissions`,
   `prework_cache_consumptions`, `prework_cache_queued_admissions`,
   `prework_cache_queued_consumptions`, `prework_cache_freshness_state`,
   `prework_cache_block_freshness_window`,
   `prework_cache_remaining_valid_blocks`, `prework_cache_hits`,
   `prework_cache_misses`, `last_prework_cache_hit`,
   `prework_cache_valid_until_processing_epoch`,
   `prework_cache_valid_until_block_sequence`,
   `last_prework_source_processing_epoch`,
   `last_prework_source_block_sequence`,
   `last_prework_admission_processing_epoch`,
   `last_prework_admission_block_sequence`,
   `last_prework_admitted_from_block_sequence`,
   `last_prework_consumption_processing_epoch`,
   `last_prework_consumption_block_sequence`, and
   `last_prework_consumed_from_block_sequence`. Queued prework may now be
   admitted against the future parameter epoch and transport state expected
   for the target block, so matching next-block control updates do not imply
   forced retirement before consumption. That transport match is defined by the
   scheduler-facing execution context (`playing`, `tempo_bpm`, and
   `timeline_position_samples`) rather than full transport-struct equality, so
   loop metadata does not widen the invalidation boundary beyond the execution
   contract. The same surface now also exposes bounded prework-queue state via
   `prework_cache_queue_capacity`, `prework_cache_queue_depth`, and
   `prework_cache_peak_queue_depth`, so queued future anticipative work is
   visible as runtime scheduler state rather than hidden behind one cache slot.
   The queue now also treats future execution state as its replacement
   boundary: identical target-state admissions may reuse an existing queued
   entry, while changed future parameter or transport targets for that same
   block are surfaced through the existing
   `SupersededByAdmission` retirement/invalidation path rather than silent
   overwrite.
   Runtime now also owns a planning-window reconciliation path for that queue,
   so future targets that are no longer part of the declared window are
   surfaced through `PlanningWindowRevised` rather than being dropped
   implicitly by host-local queue bookkeeping.
   The same runtime-owned path now also decides which future block sequences
   remain in scope for that window, so hosts no longer need to persist a
   target-block deque just to know what should be primed next. When
   anticipative prework is disabled, that planner yields an empty window, and
   when the runtime stops it retires queued prework with a runtime-owned stop
   invalidation boundary before any later restart can reuse stale future work.
   Runtime now also owns the deterministic forecast for those window targets,
   so hosts no longer construct every `RuntimePreworkWindowTarget` directly.
   Instead they declare the runtime role and runtime derives the default
   forecast profile, can switch between disabled/default/explicit/raw override
   forecast modes, keeps requested mode separate from effective mode across
   reconfigure and recovery, and derives the future transport projection,
   parameter batch, and primed input block for each planned target block
   itself. Those runtime-owned forecast transitions now also retire queued
   future prework when the forecast plan changes, but they now preserve queued
   entries that still match the revised forecast plan so stale anticipative
   work is removed without forcing a full prework flush on every forecast
   transition, and runtime immediately re-primes any missing future targets
   required by the revised plan.
   Runtime now also rebuilds that current forecast window proactively on graph
   apply and start/restart when the forecast mode and graph state allow it, so
   boot-time anticipative seeding is no longer a host-owned step.
   Runtime now also separates planned future targets from already-prepared
   future work. The exported window target list therefore represents the full
   declared future span, while the prepared queue depth can be smaller when the
   bounded prework runner has pending future targets left to drain.
   That drain path is now an explicit runtime-owned prework service lane rather
   than a side effect of current-block forecast application, so
   `engine_block_snapshot` now also carries pending-target and service-cycle
   state alongside prepared-queue state. That same snapshot now also carries
   `prework_service_state`, `prework_service_pause_count`,
   `prework_service_resume_count`, and `prework_service_starvation_count`, so
   the background prework lane is exported as a stateful runtime scheduler
   surface rather than only a bounded service call.
   It now also carries `prework_service_pressure`,
   `prework_service_throttle_count`, and `prework_service_yield_count`, so the
   shared report can describe whether runtime is draining, throttling, or
   yielding the background lane under realtime pressure instead of inferring
   that from queue depth alone.
   It also now carries backlog-aware pending-target detail:
   `prework_pending_immediate_target_count`,
   `prework_pending_near_term_target_count`,
   `prework_pending_deferred_target_count`,
   `prework_next_pending_target_block_sequence`,
   `last_prework_serviced_target_block_sequence`, and
   `last_prework_serviced_backlog_class`, so schema version 1 can distinguish
   plain queue depth from runtime scheduler prioritization under elevated
   realtime pressure.
   It now also carries `prework_service_semantic_policy`, so the shared
   report/export surface can distinguish the balanced background-lane policy
   used by the current demo graphs from latency-focused elevated-pressure
   servicing on heavier anticipative graph shapes and plugin-constrained
   servicing on mixed graphs that keep plugin-backed work on the realtime
   side of the engine boundary.
   It also carries `prework_service_active_plugin_sandboxes` and
   `prework_service_plugin_gate_active`, so the same shared export can show
   when plugin-constrained servicing has moved from “narrow the backlog” to
   “yield the background lane entirely” because multiple active plugin
   sandboxes are competing with plugin-backed realtime work.
   It now also carries bound/active/degraded/missing plugin-backed sandbox
   counts through the shared engine snapshot and planned-node export, so
   schema version 1 can expose whether plugin-constrained behavior is being
   driven by a bound active plugin session, a degraded detach path, or a
   missing binding/session match.
   Those planned-node bindings now come from the same host graph assembly
   input that declares the plugin sandbox inventory, so exported
   `planned_nodes[].plugin_sandbox_id` reflects a real host/runtime assembly
   seam rather than a separate ad hoc binding helper.
   That runtime-owned forecast mode now also owns the target window size, so
   hosts no longer choose a local anticipative horizon depth, pass a
   remaining-block cap, manage a separate forecast-advance step, or select
   local vs server forecast policy in their steady-state loop. The current
   host/runtime boundary is now: hosts declare runtime role plus current-block
   execution context, while runtime owns future-window maintenance from stored
   forecast state.
   It now also carries the cache
   lifecycle surface: `prework_cache_state`,
   `prework_cache_invalidation_count`,
   `prework_cache_retirement_count`,
   `prework_cache_unconsumed_retirement_count`,
   `prework_cache_consumed_retirement_count`,
   `last_prework_invalidation_reason`,
   `last_prework_retirement_reason`,
   `SupersededByAdmission` plus `QueueCapacityExceeded`
   invalidation/retirement reporting, and
   `last_prework_retired_unconsumed`.
4. `supervisor_report` may also carry ordered runtime event-derived sequences
   when those sequences describe runtime-owned behavior. In schema version 1,
   that includes a typed `recovery_sequence` for degraded sandbox restart
   episodes and a typed `lifecycle_sequence` for plugin-sandbox control-path
   milestones, including ensure/handshake/load/create/prepare/activate and
   transport attach/detach edges around teardown and restart. It also includes
   typed `transport_sequence` and `heartbeat_sequence` paths for broker churn
   and control-loop markers, plus `block_dispatch_sequence` and
   `lease_rollover_sequence` paths for brokered render work and lease
   generation changes, plus `invalidation_sequence` for explicit completion
   and lease invalidation boundaries, plus `completion_slot_sequence` for
   exact completion-slot and fallback-application transitions, plus a
   canonical top-level `transport_fault_sequence` with explicit source,
   phase, resource, and operation metadata, including projected timeout,
   detach-lifecycle, invalidation, and fallback boundaries where those
   transitions matter to transport-fault supervision, plus
   `broker_failure_sequence` for typed transport-side failure points, plus
   `sandbox_operation_failure_sequence` for typed sandbox-emitted attach,
   flush, and protocol failures.
5. `supervisor_report.transport_fault_summary` is the canonical boundary
   declaration for that top-level transport fault view. In schema version 1,
   it freezes the boundary mode as `FaultAdjacentOnly`.
6. `supervisor_report.transport_session_summary` is the healthy-path
   companion to that fault view. It exists so transport/session visibility can
   grow without weakening the explicit `FaultAdjacentOnly` meaning of
   `transport_fault_sequence` and `transport_fault_summary`. In schema version
   1 it also carries current attachment state, heartbeat freshness, dispatch
   state, attached-session concurrency counts, per-session liveness inside
   `active_sessions`, per-session transport-fault freshness/history derived
   from the canonical transport-fault stream, plus active or last
   sandbox/lease/region identity. This surface is now frozen as stable for
   schema version 1.
7. Runtime transport-session admission policy is intentionally a separate
   control-layer concern from that schema surface. Steady-state versus
   recovery-overlap attach admission lives in runtime-owned control state and
   may evolve without widening `transport_session_summary`. The current host
   recovery path now uses that policy during a real overlapping broker-session
   handoff rather than only around detach/reattach sequencing, and the overlap
   path now includes rollback of the replacement session if old transport
   teardown or replacement startup fails mid-handoff. Runtime admission now
   rejects a second concurrent `RecoveryOverlap` attach until the prior overlap
   session is torn down, and rejected overlap prepares must clean their broker
   region plus lifecycle state back up before returning the admission error.
   Recovery orchestration is also now allowed to re-enter while runtime is
   already stopped from a prior degraded attempt, so staged retry logic can
   drive interleaved teardown-fault and overlap-admission failures without
   first forcing a full runtime restart. Runtime-owned
   `transport_concurrency_snapshot` now also carries lingering detach state
   (`current_lingering_sessions`, `current_detach_requested_sessions`,
   `current_detach_faulted_sessions`, and per-session `state`) so detach
   latency and failed old-session teardown are represented as first-class
   control state instead of only as host-local deferred failure behavior. Host
   recovery control may now also consume that runtime-owned lingering state to
   perform a later explicit teardown cleanup and fresh restart without widening
   the schema surface or forcing a full runtime reset. That cleanup path may
   itself fail and be retried later while the lingering session remains visible
   in runtime admission state. Host recovery now also sweeps orphan lingering
   sessions for the same sandbox before a fresh overlap attach, so failed
   replacement rollback state does not silently consume recovery-overlap
   capacity on the next attempt. If that orphan cleanup lacks valid
   runtime-owned transport metadata, recovery aborts rather than skipping the
   stale lingering session. That cleanup path now also supports more than one
   orphan lingering session candidate for the same sandbox instead of assuming
   a single replacement-side leftover. After replacement attach has already
   succeeded, hosts may also perform late lingering-session reconciliation as a
   post-start cleanup path; that reconciliation must preserve the active
   replacement session when it succeeds, and must leave lingering state visible
   rather than stopping the recovered runtime if cleanup fails. Host control
   now also treats lingering cleanup as two explicit modes: strict pre-attach
   cleanup, which may abort the next recovery handoff, and best-effort
   post-start reconciliation, which may leave lingering state visible while the
   active replacement session stays live. Adjacent recovery episodes must honor
   that same split so an older lingering origin is either swept before the next
   overlap handoff or blocks it cleanly without widening runtime transport
   admission beyond the current policy.
   Lingering-session provenance and cleanup ordering are now runtime-owned as
   well: `transport_concurrency_snapshot.active_sessions` records
   `provenance`, `attach_sequence`, and `attach_processing_epoch`, and hosts
   consume runtime-produced cleanup candidates instead of sorting origin and
   replacement lingerers locally. Those active lingering sessions now also
   carry runtime-owned cleanup scheduling state: `cleanup_attempt_count`,
   `last_cleanup_mode`, `cleanup_in_progress`, `last_cleanup_epoch`, and
   `last_cleanup_error`. Runtime now also exposes a typed lingering cleanup
   workflow boundary: hosts request a `LingeringCleanupPlan` for one sandbox
   and report cleanup success/failure back through runtime APIs rather than
   open-coding cleanup loops against the snapshot directly. Runtime now also
   owns queued cleanup work and deferred retry issuance for that same boundary,
   and `transport_concurrency_snapshot.pending_cleanup_work_items` exposes the
   queued cleanup pressure in the shared report/export surface. Runtime now
   also owns cleanup epochs and retry readiness for that queued work, and the
   shared snapshot/export includes deferred-retry count, next cleanup epoch,
   and the oldest pending cleanup ready epoch. Runtime now also groups queued
   cleanup into sandbox-scoped cleanup waves: `cleanup_wave` is carried on
   cleanup plans and active sessions, and
   `transport_concurrency_snapshot.pending_cleanup_waves` exposes per-wave
   queue summaries so one late-detach cleanup cycle can be distinguished from
   the next without relying only on global cleanup-epoch ordering.
8. `host_summary` is an assembly-level supplement for profile-specific or
   profile-shaped fields that are not part of the shared runtime report.
9. `host_summary` should not mirror runtime-owned readiness, diagnostics, or
   supervision state; those belong in `supervisor_report`.
9. Timeline continuity belongs in `supervisor_report`, not only in
   host-specific summaries.
10. Automation continuity belongs in `supervisor_report` through the shared
   `RuntimeAutomationSnapshot` surface.
11. `host_summary` should not mirror runtime-owned automation continuity fields.
   If a continuity field is needed outside `supervisor_report`, it must be
   justified as an assembly-local convenience rather than copied by default.
12. `host_summary` should not mirror runtime-owned block-sequence continuity
   fields either; sequence segments, epochs, gaps, and rollover counts belong
   in `supervisor_report`.
13. `host_summary` should not mirror runtime-owned automation counters or
    automation value snapshots either; those belong in
    `supervisor_report.automation`.
14. Assembly-local payload outcomes may remain in internal host summaries for
    tests and debugging, but they are not part of the default exported
    `host_summary` contract in schema version 1.
15. `signal-supervisor-tools` may expose payload detail only through an
    explicit opt-in debug path such as `--include-payload`; that opt-in adds a
    grouped `payload` block without changing the meaning of the default export.
16. Assembly-local control, transport, and fault detail should follow the same
    rule: grouped host-local execution blocks are preferred over flat summary
    fields when those details do not belong in `supervisor_report`.
17. Host-issued recovery detail may appear inside the grouped `execution`
    block when it explains assembly behavior rather than attempting to replace
    the shared runtime report. In schema version 1, that exception is limited
    to fields such as the last host recovery intent and the stop reason the
    host requested for degraded restart handling.
18. `host_summary` should declare which grouped sections are present through a
    stable `sections` list so automation can distinguish default and opt-in
    debug exports without inferring intent from missing keys alone.
19. `host_summary` should also declare `debug_sections_supported` and
    `debug_sections_enabled` so the current targeted debug policy is visible in
    the export itself, not only in documentation.
20. The preferred grouped `host_summary` shape is:
   - top-level identity/profile fields only
   - `sections`
   - `debug_sections_supported`
   - `debug_sections_enabled`
   - `execution`
   - `transport`
   - `faults`
21. When payload detail is explicitly requested, it should appear as one
    grouped `payload` block rather than as top-level counter sprawl.
22. The current targeted debug-section model supports only `payload`; adding
    any new opt-in section requires an explicit contract and implementation
    batch rather than silent flag growth.
23. Schema evolution must be deliberate:
   - additive fields may extend `schema_version = 1` if existing fields keep
     their meaning
   - breaking shape changes require a new `schema_version`
24. `signal-supervisor-tools --describe-export` is the canonical host-free
    introspection path for the frozen schema version, default host-summary
    sections, and supported debug sections.

## Placement Decision

- Block-sequence continuity is runtime-owned and belongs directly in
  `supervisor_report`.
- Automation continuity is now also runtime-owned and belongs in
  `supervisor_report` through `RuntimeAutomationSnapshot`.

## Acceptance Signals

- `signal-supervisor-tools --format=json ...` emits the versioned envelope.
- `signal-supervisor-tools --format=json ...` emits grouped `host_summary`
  blocks instead of a flat execution-field dump.
- `signal-supervisor-tools --format=json ...` exposes the default grouped
  section list through `host_summary.sections`.
- `signal-supervisor-tools --format=json ...` may include host-issued recovery
  detail such as `host_summary.execution.last_recovery_intent` and
  `host_summary.execution.last_stop_reason` without turning `host_summary` into
  a generic mirror of runtime control state.
- `signal-supervisor-tools --format=json ...` exposes runtime-owned degraded
  restart tracing through `supervisor_report.recovery_sequence`.
- `signal-supervisor-tools --format=json ...` exposes runtime-owned sandbox
  control-path tracing through `supervisor_report.lifecycle_sequence`.
- `signal-supervisor-tools --format=json ...` exposes handshake/load/create,
  prepare/activate, and transport attach/detach milestones through that
  `lifecycle_sequence` path instead of requiring tooling to infer them from
  aggregate state.
- `signal-supervisor-tools --format=json ...` exposes runtime-owned broker
  attach/detach and detach-fault tracing through
  `supervisor_report.transport_sequence`.
- `signal-supervisor-tools --format=json ...` exposes runtime-owned heartbeat
  request/response/miss tracing through
  `supervisor_report.heartbeat_sequence`.
- `signal-supervisor-tools --format=json ...` exposes runtime-owned block
  dispatch request/completion/timed-out tracing through
  `supervisor_report.block_dispatch_sequence`.
- `signal-supervisor-tools --format=json ...` exposes runtime-owned lease
  generation changes through `supervisor_report.lease_rollover_sequence`.
- `signal-supervisor-tools --format=json ...` exposes runtime-owned completion
  and lease invalidation milestones through
  `supervisor_report.invalidation_sequence`.
- `signal-supervisor-tools --format=json ...` exposes runtime-owned completion
  slot readiness/processing/completed/timed-out/invalidated/fallback-applied
  milestones through `supervisor_report.completion_slot_sequence`.
- `signal-supervisor-tools --format=json ...` exposes the canonical runtime-
  owned top-level transport fault view through
  `supervisor_report.transport_fault_sequence`, with `source`, `phase`,
  `resource`, and `operation` fields that distinguish host-broker failures
  from sandbox-operation failures, and with projected runtime-dispatch
  milestones for timeout, detach lifecycle, explicit invalidation, and
  fallback application where those transitions are part of the transport-fault
  story.
- `signal-supervisor-tools --format=json ...` exposes
  `supervisor_report.transport_fault_summary` so tooling can see that the
  canonical top-level view is `FaultAdjacentOnly` without inferring that
  policy from the absence of healthy-path markers.
- `signal-supervisor-tools --format=json ...` exposes
  `supervisor_report.transport_session_summary` so tooling can inspect healthy
  attach/detach, heartbeat, and dispatch activity plus current attachment
  state, heartbeat freshness, dispatch state, attached-session concurrency,
  active identity, per-session liveness, and per-session fault freshness
  without broadening the canonical fault surface.
- `signal-supervisor-tools --format=json ...` exposes runtime-owned transport-
  side failures such as prepare-plan creation, payload write/read, broker
  destroy, and transport teardown through
  `supervisor_report.broker_failure_sequence`.
- `signal-supervisor-tools --format=json ...` exposes sandbox-emitted attach,
  flush, and protocol failures derived from CLAP harness fault envelopes
  through `supervisor_report.sandbox_operation_failure_sequence`.
- `signal-supervisor-tools --format=json ...` exposes the current debug policy
  through `host_summary.debug_sections_supported` and
  `host_summary.debug_sections_enabled`.
- `signal-supervisor-tools --describe-export --format=json` exposes the frozen
  schema/version and supported debug-section policy without running a host.
- `signal-supervisor-tools --format=json ...` does not export host-local
  payload detail by default.
- `signal-supervisor-tools --format=json --include-payload ...` may add one
  grouped `payload` block for explicit debugging, and `host_summary.sections`
  expands accordingly.
- no other opt-in debug section is currently supported.
- `RuntimeSupervisorReport` exposes timeline continuity directly.
- `RuntimeSupervisorReport` exposes automation continuity directly through
  `RuntimeAutomationSnapshot`.
- Docs point to this contract when describing supervisor export behavior.

## Next Task

Harden lingering-session race handling around late detach completion, especially
when a previously faulted origin teardown resolves after a fresh replacement
attach and the host needs to fold that completion back into runtime admission
without disturbing the active replacement session.
