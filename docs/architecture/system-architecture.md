# System Architecture

Status: active
Owner: core-product
Updated: 2026-07-25
Vision refs: `docs/vision/001-signal-vision.md`

## Top-Level Stack

Signal owns the shared audio-systems stack used by Loophole, Finch, and future
apps.

The intended top-level layers are:

1. `signal-primitives`
   - audio/sample/time/channel primitives
   - realtime-safe math and utility types
2. `signal-dsp`
   - reusable DSP kernels and transforms
   - smoothing, metering, resampling, filters
3. `signal-analysis`
   - onset, beat, tempo, tonal, loudness, and future embedding-related analysis
   - reusable offline and streaming analysis logic
4. `signal-graph`
   - graph execution semantics
   - routing, latency/tail accounting, parameter-event application
5. `signal-runtime`
   - embeddable runtime orchestration
   - diagnostics, scheduling, lifecycle, and host-facing runtime state
   - runtime-owned executable graph/block processing state and metrics
6. host-edge adapters
   - plugin-format adapters
   - hardware/device adapters
   - narrow FFI or IPC boundaries only where platform reality forces them

The current package-level naming proposal is recorded in
`docs/architecture/package-map.md`.

## Data and Authority Flow

- Signal owns audio execution, DSP, analysis, graph/runtime semantics, and
  runtime diagnostics.
- Pulse remains the authority for Loophole project/session state and editing.
- Finch remains the authority for app workflow, review UX, sidecar handling, and
  library-specific behavior.
- Finch and Loophole both consume Signal-owned crates or runtime surfaces rather
  than reimplementing core analysis logic locally.

## Invariants

- Reusable DSP and analysis do not live in Finch-local or Loophole-local wrapper
  code.
- Plugin and hardware integrations must not become the home of core DSP logic.
- Process boundaries follow trust and stability needs, not historical repo
  layout.
- Supervisor export contracts prefer shared runtime report surfaces over
  host-specific summary duplication.
- Real-time paths avoid blocking, allocation churn, and unbounded work.
- Research authority for DSP and analysis topics lives in `docs/research/`.

## Time-Stretch Boundaries

- `OfflineHighQuality` is the frozen transparent production route. Contract
  `084` is closed without successor promotion.
- `CreativeStretch` is a separate public offline whole-buffer API for exact
  `4x`, `8x`, and `16x` neutral `Dream`, with `space` as its only adjustable
  creative control.
- The admitted private `Cyclic` renderer owns exact `2x`, `4x`, and `8x`.
  Its public extension exposes one `5..90 ms` cycle duration.
- `DirectRenewalDream` remains an internal renderer identity. The public API
  fixes its admitted seed and never falls back to the transparent renderer.
- Creative cache, artifacts, automatic routing, dynamic ratio, runtime
  integration, Loophole, and Chorus remain outside the admitted boundary.
- `RealtimePreview` remains separate and unsupported as a direct audio-thread
  source until its source-fill contract reopens.

## Supervisor Export Boundary

- `signal-runtime` owns the shared supervisor report types and any continuity
  state that has been promoted into runtime-owned surfaces.
- Host assemblies may expose convenience summaries, but runtime-owned continuity
  state should be sourced from `signal-runtime` rather than recomputed locally.
- `signal-supervisor-tools` is the versioned export boundary for machine-readable
  soak and restart reporting.
- The explicit export and report rules live in
  `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`.
- Recovery is moving from “tear down then rebuild” toward “overlap then hand
  off”: replacement broker sessions can be admitted under runtime-owned
  recovery-overlap policy before old transport teardown completes, and the
  overlap path now rolls replacement sessions back if teardown or startup fails
  mid-handoff. Runtime now also rejects a second competing overlap attach until
  the first overlap session has been torn down and cleans rejected overlap
  prepares back up immediately. Recovery orchestration can now re-enter while
  runtime is already stopped from a prior degraded attempt, which supports
  staged retry paths. Detach latency is now represented as runtime-owned
  lingering-session control state as well: `DetachRequested` and
  `DetachFaulted` sessions remain visible in `transport_concurrency_snapshot`
  until teardown actually completes, so recovery admission and inspection do
  not rely only on host-local deferred failure injection. Subsequent degraded
  recovery attempts can now explicitly clean those lingering sessions up and
  restart from a fresh transport attach after the old session is finally torn
  down, rather than requiring a full runtime reset to clear admission state.
  That explicit cleanup path is now also retryable: if the first cleanup retry
  fails, the session remains faulted and visible in runtime admission until a
  later recovery attempt succeeds. Recovery now also sweeps orphan lingering
  sessions for the same sandbox before a fresh overlap attach, so failed
  replacement rollback state does not keep consuming recovery-overlap capacity
  across later attempts. If that orphan cleanup cannot reconstruct a valid
  transport boundary from runtime-owned metadata, recovery aborts rather than
  quietly stepping past stale lingering state. That sweep now handles more than
  one orphan lingering session candidate for the same sandbox, so the recovery
  path does not assume a single stale replacement-side transport. After a fresh
  replacement session is already attached and running, late origin teardown
  completion is now reconciled as post-start cleanup against the same
  runtime-owned lingering-session state: successful late cleanup frees the
  lingering slot without disturbing the active replacement session, and failed
  late cleanup leaves the lingering session visible instead of tearing the
  recovered runtime back down. Recovery now also distinguishes strict
  pre-attach lingering cleanup from best-effort post-start reconciliation, so
  stale lingering state either blocks the next overlap handoff up front or is
  carried forward visibly while the recovered session remains live. That split
  is now exercised across adjacent recovery episodes as well: a prior lingering
  origin can be swept before the next overlap recovery from a newer lingering
  replacement, and invalid stale metadata aborts that adjacent recovery instead
  of creating a third transport session or silently skipping the stale state.
  Runtime now also owns lingering-session provenance and cleanup candidate
  ordering for that state: `transport_concurrency_snapshot.active_sessions`
  records whether a lingering session is a steady origin or recovery
  replacement, plus attach order and attach epoch, and host cleanup paths
  consume a runtime-produced candidate list instead of re-sorting lingerers
  locally. Runtime now also tracks lingering cleanup scheduling state for each
  active lingering session, including cleanup mode, attempt count,
  cleanup-in-progress, last cleanup epoch, and last cleanup error, so strict
  pre-attach cleanup and best-effort post-start reconciliation are visible in
  shared runtime transport state. Hosts now consume that through an explicit
  runtime cleanup workflow surface: runtime produces a `LingeringCleanupPlan`
  for one sandbox and hosts report cleanup success or failure back through
  runtime-owned APIs instead of coordinating cleanup as raw loops over snapshot
  state. Runtime now also owns pending cleanup work and deferred retry
  scheduling for that workflow, so late-detach cleanup pressure is represented
  as queued runtime work (`pending_cleanup_work_items`) rather than only as
  host-local decisions about when another cleanup pass should run. That queued
  work now also carries runtime-owned cleanup epochs and ready-at scheduling,
  so deferred retries back off in processing-epoch space instead of being
  drained immediately after requeue. Runtime now also groups that queued work
  into sandbox-scoped cleanup waves, so one lingering cleanup cycle can be
  distinguished from the next in shared runtime state: `cleanup_wave` travels
  with queued cleanup work, cleanup plans, and active sessions, while
  `transport_concurrency_snapshot.pending_cleanup_waves` exposes one summary
  per sandbox/wave in the supervisor surface.
  Signal now also has a concrete execution slice in this same runtime layer:
  `signal-runtime` owns an executable graph projection, processes stage-based
  audio blocks through `signal-graph`, and exposes runtime-owned
  `engine_block_snapshot` metrics. Both the local and server hosts now run
  that graph path in their block loops, so the Rust engine work is no longer
  only lifecycle and recovery policy.
  That execution slice now has an explicit runtime-owned execution context:
  projection epoch, parameter epoch, configured block size, and transport
  state are attached to each engine block, and transport position advances in
  runtime after graph processing instead of being implicit in host-side block
  generation. It also now uses a node/plan projection model instead of one
  flat stage chain, so `signal-runtime` is starting to own a more
  scheduler-facing execution structure. That node/plan model now includes
  basic execution classes and latency metadata, so the runtime can distinguish
  pure transforms from stateful and latency-bearing nodes in its shared engine
  snapshot. Runtime now also converts those hints into a planning view with
  inline-realtime, stateful-realtime, and anticipative-eligible groups, so the
  shared engine snapshot describes how the current graph would be executed
  under the active anticipative mode rather than only exposing raw node
  categories. That is now reflected in real graph execution structure too:
  runtime passes the active anticipative mode into the graph execution
  context, and `signal-graph` executes nodes by derived planning phases rather
  than only the original flat node list. Those phases are now grouped into
  explicit anticipative and realtime lanes, so the runtime-owned engine
  surface has the beginnings of a scheduler boundary rather than only a richer
  block processor. Runtime now also executes an explicit dispatch sequence
  derived from those lanes, so anticipative and realtime work exist as a
  concrete engine policy even though execution is still single-threaded. That
  dispatch policy now has a real handoff inside the engine: the anticipative
  dispatch produces a prepared prework buffer first, then the realtime
  dispatch consumes that prepared result, so the runtime now owns an explicit
  preparation boundary instead of only dispatch ordering metadata. That
  prepared result is now also cached in runtime for a short validity window,
  so adjacent blocks with matching graph/projection/parameter/input state can
  reuse anticipative prework instead of regenerating it immediately. That
  runtime-owned prework cache now also distinguishes inline admission from
  later queued admission and later consumption, so scheduler-facing engine
  state can tell whether prepared work was merely admitted for reuse, queued
  for a future block, or actually consumed by a later block.
  Freshness is now tracked in block-sequence space, so the runtime can expose
  when prepared work is still fresh, expiring, or exhausted on real block
  progression instead of inferring freshness only from processing epochs.
  That
  cache is now invalidated by real control-path changes as well: runtime
  reconfigure, graph changes, transport changes, parameter batches, expiry,
  and input-signature mismatches all retire stale prework through runtime
  state rather than leaving cache invalidation implicit in the next block.
  Queued prework admission can now also be bound to the future parameter epoch
  and transport state expected for the target block, so matching next-block
  control updates do not automatically invalidate queued work before runtime
  has a chance to consume it.
  The host execution paths now also make transport progression explicit per
  block: local and server apply deterministic block-start transport
  projections, and queued prework is admitted against the next block's
  expected transport state instead of relying only on implicit post-block
  transport advancement inside runtime. Runtime invalidation uses the same
  transport fields as prework matching, so loop metadata does not accidentally
  retire reusable work.
  That scheduler surface is now backed by a bounded runtime-owned prework
  queue rather than a single prepared-result slot. Runtime can carry more than
  one future block of anticipative work, exposes queue depth in the shared
  engine snapshot, and evicts the oldest future entry explicitly when queued
  prework exceeds capacity.
  That same queue now uses future execution state as its replacement boundary:
  identical future-block admissions reuse the existing queued entry, while a
  changed future parameter or transport target for the same block explicitly
  retires the older queued entry as `SupersededByAdmission`.
  Local and server host assemblies now keep a small primed future-block
  horizon, so runtime-owned prework is exercised as a real multi-block
  scheduler path rather than only one-block lookahead.
  That priming path is now routed through a runtime-owned planning window API
  instead of repeated host-local single-block admissions, so runtime can
  explicitly retire queued targets that have fallen out of the declared future
  window with `PlanningWindowRevised`.
  Runtime now also owns the future block-sequence planning for that window,
  so hosts only supply future state for the block sequences runtime says are
  still in scope. When anticipative prework is off, that planner returns no
  target sequences, and when the runtime stops for degraded recovery it
  retires queued prework before restart rather than carrying stale future work
  into the new epoch.
  Runtime now also owns the deterministic forecast used to prime that window.
  Local and server hosts only declare runtime role plus current execution
  context, while runtime derives the default forecast profile for that role,
  can switch between disabled/default/explicit/raw override forecast modes,
  keeps requested mode separate from effective mode, and then derives the
  future transport projection, parameter batch, and primed input block for
  each planned target block itself. Those runtime-owned mode/profile/policy
  transitions are now also scheduler invalidation boundaries, so queued future
  prework is reconciled when the future plan changes rather than leaking
  across incompatible forecast state. Compatible queued entries are preserved,
  while only out-of-scope future work is retired, and runtime immediately
  re-primes any missing future blocks that remain inside the revised window.
  That same runtime-owned scheduler path now proactively rebuilds the current
  forecast window on graph apply and start/restart when enough runtime state is
  available, so host assemblies no longer need a special boot-only forecast
  priming step to seed anticipative work.
  Runtime now also distinguishes planned future targets from prepared future
  work. The forecast window can therefore be larger than the prepared prework
  queue, with runtime draining pending future targets through a bounded
  anticipative service cycle instead of preparing the whole window at once.
  That service cycle is now a real runtime-owned lane boundary rather than an
  incidental side effect of forecast application: hosts apply only the current
  block forecast state, while runtime advances pending future targets through
  an explicit prework service call around the realtime cycle.
  That lane now also exposes explicit runtime scheduler state:
  `Disabled`, `Idle`, `Pending`, `Servicing`, `Paused`, and `Starved`, plus
  pause/resume/starvation counters in the shared engine snapshot.
  It also now has an explicit pressure boundary. Hosts can hint `Normal`,
  `Elevated`, or `Critical` realtime pressure, and runtime will throttle or
  yield the anticipative background lane accordingly instead of servicing it
  identically under timeout or watchdog pressure.
  That pressure path now also has a runtime-owned backlog policy: pending
  future targets are classified as immediate, near-term, or deferred work,
  and elevated pressure only services the nearer backlog classes while
  deferred targets remain queued for later background service.
  That scheduler boundary is now also informed by graph semantics. Runtime
  derives a prework service semantic policy from the current graph planning
  shape, and latency-focused anticipative graphs can retain a wider elevated-
  pressure service scope than balanced graphs instead of all graphs sharing
  one fixed backlog cutoff.
  Mixed graphs with plugin-backed realtime nodes now narrow that same policy
  into a plugin-constrained path, so elevated pressure keeps background
  servicing on immediate future work instead of widening scope across the full
  anticipative backlog.
  That plugin-constrained path is now also runtime-aware of active plugin
  sandbox count. The shared engine snapshot carries
  `prework_service_active_plugin_sandboxes` and
  `prework_service_plugin_gate_active`, and runtime can fully yield the
  anticipative service lane when multiple active plugin sandboxes are present
  under non-normal pressure instead of letting hosts invent their own
  plugin-heavy scheduler exceptions.
  That same path now also accepts plugin-backed node bindings as runtime
  projection input, so plugin-heavy scheduler policy can be derived from the
  live transport-session state of the specific sandbox a realtime plugin node
  is bound to rather than only a global active-sandbox count.
  In the current host assemblies, those bindings are carried alongside the
  graph projection and plugin sandbox inventory as one assembly description,
  so runtime gets plugin-backed ownership from the same host-side graph
  assembly seam that instantiates the sandboxes.
  That same runtime-owned forecast mode also owns the prework window size, so
  hosts no longer clamp a local horizon constant, pass a remaining-block cap,
  manage a separate forecast-advance step, or choose local vs server forecast
  policy in their boot path. They apply only the current execution context,
  and runtime reconciles how many future blocks stay inside the anticipative
  window automatically.
  Runtime also now records whether those retirements happened before any
  future consumption or only after the prepared work had already been used.
  staged retry logic across interleaved teardown and admission failures.

## Performance and Reliability Constraints

- Realtime-safe code paths must be deterministic and allocation-aware.
- Shared crates must remain usable in both runtime and offline-analysis
  contexts.
- Plugin hosting is treated as untrusted by default; sandboxing remains the
  preferred containment layer.
- Native shims are acceptable where ABI or platform constraints make them the
  lower-risk integration choice.

## Interfaces With Roadmaps

- `g10.030` closes transparent stretch successor work on the frozen
  `OfflineHighQuality` baseline.
- `g10.031` publicly admits creative `Dream`; `g10.033` widens it to every
  exact target in `4x..16x` through one owner without a same-character router.
- `g10.032` owns `Cyclic` separately; `g10.034` widens its public exact-target
  domain to every target `2N..=8N` without changing the admitted renderer.
- `g10.035` selects one future opt-in Automatic intent. Transparent owns
  through `4N`, Transparent and neutral Dream transition over `4N..=8N`, and
  Dream owns `8N..=16N`. Cyclic remains explicit. The complete private route
  brief is corrected after an evidence-invalid first checkpoint; one exact
  replay is ready.

## Next Task

Execute `g10.035` Batch 35.5 only. Restore and hash-prove the exact Batch 35.3
`ExactTargetTransparentDreamRouter` source in its newly named disposable
worktree, pass conformance twice, and restart the corrected gate at
identity/parity. Keep all public, runtime, cache, artifact, UI, Loophole, and
Chorus work blocked.
