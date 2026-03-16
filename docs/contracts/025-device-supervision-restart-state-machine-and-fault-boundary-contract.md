# 025 Device Supervision, Restart-State Machine, And Fault-Boundary Contract

Status: complete
Owner: core-product
Updated: 2026-03-15
Related contracts: `docs/contracts/006-runtime-hardware-portability-and-clock-domain-contract.md`, `docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md`, `docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`, `docs/architecture/package-map.md`

## Purpose

Freeze the first runtime-owned device supervision and restart-state contract for
`g06.014` so later hardware recovery, clocking, monitoring, and external-I/O
depth can widen on top of one reusable supervision substrate instead of
reopening host-local restart policy, backend-specific fault labels, or device
loss heuristics.

## Authority hierarchy

Device supervision and restart-state meaning have one authority chain:

1. `signal-hardware` owns backend-neutral hardware capability and diagnostic
   primitives for:
   - device identity and negotiated stream contracts
   - lifecycle ownership and restart policy
   - backend health, device-loss, restart-attempt, and restart-failure counters
2. `signal-runtime` owns canonical supervision and fault-boundary meaning for:
   - `RuntimeSupervisionSnapshot`
   - `RuntimeFaultStatusSnapshot`
   - `RuntimeInterruptionSummary`
   - `RuntimeDegradationSummary`
   - `RuntimeFaultDiagnosticReceipt`
   - runtime observation and supervisor export surfaces that summarize recovery
     state, safe mode, watchdog activity, and restart exposure
3. host crates may broker backend callbacks, restart attempts, and hardware
   diagnostics into runtime-owned state, but they must not become the authority
   for:
   - restartable versus exhausted recovery classification
   - faulted versus recovering device state
   - competing device-loss or restart taxonomies outside the runtime contract

If a device supervision or restart claim cannot be explained through
`signal-hardware`, `signal-runtime`, and additive shared receipts, it is not
yet part of the reusable Signal contract.

## Existing runtime anchors

This contract is grounded in the current runtime-owned and host-fed supervision
surface family:

- `RuntimeSupervisionSnapshot`
- `RuntimeFaultStatusSnapshot`
- `RuntimeInterruptionSummary`
- `RuntimeDegradationSummary`
- `RuntimeFaultDiagnosticReceipt`
- `RuntimeHostIoSummary`
- `RuntimeHostClockingSummary`
- `RuntimeHostObservationReport`
- `RuntimeHostSupervisorReport`
- `RecoveryRecord`
- `HardwareDiagnosticsSnapshot`
- `HardwareLifecycleContract`
- `HardwareRestartPolicy`

Batch 14.1 does not claim these anchors already form a complete supervision
state machine. It freezes how later DTOs and receipts must deepen from this
existing surface family.

## Shared vocabulary

### Device supervision

`device supervision` means the runtime-owned interpretation of backend health,
restart policy, restart attempts, restart failures, safe mode, and device-loss
state for the active hardware path.

It is not the same thing as raw backend diagnostics. Backend counters and host
callbacks are evidence that runtime surfaces consume, not a competing consumer
boundary.

### Restart episode

`restart episode` means one bounded recovery attempt against the active device
path after a device loss, watchdog-triggered restart, or equivalent hardware
continuity break.

A restart episode may include:

- detection of the interruption
- one or more backend or host restart attempts
- either return to a healthy device path or escalation into exhausted or
  faulted state

Restart episodes are runtime-owned continuity events even when hosts execute
the mechanical restart steps.

### Exhaustion

`exhaustion` means the runtime has consumed the currently allowed recovery path
for the active hardware failure and cannot keep presenting the device state as
recovering.

Exhaustion is stronger than “one restart failed.” It is the explicit boundary
where the runtime stops treating the hardware path as plausibly restartable
within the current episode and must surface an escalated fault state.

### Fault boundary

`fault boundary` means the line where Signal stops describing a device issue as
recovering or restartable and instead exports an explicit faulted or terminal
hardware outcome.

The fault boundary must remain aligned with:

- the interruption contract from `012`
- runtime fault-cause attribution from `016`
- later clock-domain and endpoint-topology work that depends on supervision
  state without redefining it

### Recovering hardware state

`recovering hardware state` means the runtime still considers the active device
path interruptive but plausibly recoverable.

Recovering hardware state may include:

- active device loss
- watchdog-triggered restart work
- restart attempts still in progress
- safe mode or reduced runtime behavior while recovery continues

Recovering state must stay distinct from a fully faulted or exhausted device
path.

### Faulted hardware state

`faulted hardware state` means the runtime has crossed the hardware fault
boundary for the active episode and no longer represents the device path as
recovering successfully.

Faulted state may still preserve diagnostics and restart history, but products
must not treat it as equivalent to a temporary restartable interruption.

## Rules

### Rule 1: runtime supervision remains authoritative for recovery classification

Products and hosts must not infer restartable versus exhausted hardware state
from backend restart counters, callback loss, or host-local recovery scripts
alone.

Runtime observation and supervisor receipts remain the shared authority for:

- whether hardware state is steady, recovering, or faulted
- whether device loss remains active
- whether safe mode and watchdog activity are part of the current episode

### Rule 2: backend diagnostics are evidence, not a competing taxonomy

`HardwareDiagnosticsSnapshot` and host-fed counters remain important evidence,
but they do not become a second consumer-facing fault model.

Consumers should not need to reinterpret:

- `device_loss_count`
- `restart_attempt_count`
- `restart_failure_count`
- callback overrun counts

just to decide whether the runtime considers the device path recoverable.

### Rule 3: exhaustion must remain explicit

Later runtime DTOs must preserve a direct way to observe that the current
hardware recovery path is exhausted rather than forcing consumers to guess from
restart counts or a failed boot alone.

That distinction matters because later monitoring, loopback, and external-I/O
work need one stable answer for:

- temporary restart activity
- degraded but still running state
- explicitly faulted hardware state

### Rule 4: device loss and watchdog restart stay aligned but distinct

This contract intentionally aligns hardware supervision to the broader
interruption and fault contracts, but it does not collapse every restart cause
into one undifferentiated restart bucket.

Later runtime surfaces must preserve enough detail to distinguish:

- device-path loss or restart failure
- watchdog-driven restart or safe-mode entry
- broader runtime degradation that is not a hardware fault boundary

### Rule 5: later clocking and external-I/O work must build on supervision state

`g06.015` and later hardware/media milestones may widen drift, duplex mismatch,
endpoint topology, monitoring, and loopback detail, but they must not redefine
restart-state or hardware fault ownership.

The supervision substrate defined here stays the base layer those later
contracts compose with.

## Deferred scope

Batch 14.1 intentionally keeps the following outside the shared contract:

- exhaustive backend certification or hardware compatibility matrices
- device-setup UX, picker flows, or product-local recovery prompts
- remote or distributed hardware supervision
- network-audio restart behavior
- control-surface or external MIDI device policy
- clock drift, duplex mismatch, and endpoint-topology semantics beyond the
  already-frozen portability boundary from `006`

Those areas may later gain additive Signal-owned surfaces, but they are not
promised by Batch 14.1.

## Batch 14.1 outcome

Batch 14.1 freezes the first bounded device supervision and restart-state
contract:

- Signal now has one shared vocabulary for restart episodes, recovering state,
  exhaustion, and hardware fault boundaries instead of leaving those meanings
  implicit in host restart loops or backend counters
- backend diagnostics and host callbacks are explicitly frozen as contributing
  evidence for runtime supervision rather than a competing consumer taxonomy
- later runtime DTOs can now deepen supervision, restart, and exhaustion
  receipts against one fixed authority chain
- `g06.015` and later hardware lanes now have a stable supervision substrate to
  build on instead of reopening restart ownership during clocking or
  endpoint-topology work

## Batch 14.2 outcome

Batch 14.2 materializes the first runtime-owned device supervision receipt
family on top of this contract:

- `signal-runtime` now exports a bounded
  `RuntimeDeviceSupervisionSnapshot` through runtime observation and supervisor
  report surfaces
- the shared snapshot carries runtime-owned classification for:
  - supervision state (`Stable`, `Recovering`, `Exhausted`, `Faulted`)
  - restart-state progression (`Unneeded`, `Attempting`, `Recovered`,
    `Exhausted`, `Faulted`)
  - the hardware fault boundary
  - aligned interruption and recovery-state context
- host-fed hardware evidence remains additive rather than authoritative:
  - device-loss counts
  - restart attempts and restart failures
  - watchdog restart activity
  - restart policy, backend health, stream state, and active device identity
- `signal-host-local` now enriches the shared runtime-owned report family with
  host I/O evidence instead of requiring host-private supervision
  reconstruction
- the focused proof spine now covers recovered and exhausted device-loss
  episodes on the shared observation and supervisor-report boundary

Batch 14.2 still stops short of full closure: explicit faulted-device proof and
stronger shared consumer acceptance remain Batch 14.3 work.

## Batch 14.3 outcome

Batch 14.3 closes the first bounded device-supervision proof boundary:

- `signal-runtime` now has a downstream-style public proof for recovered
  episode history plus explicit faulted-device supervision state
- `signal-host-local` proves recovered, exhausted, and explicit faulted device
  outcomes on the stable `supervisor_report()` edge
- `signal-host-server` proves the stable host edge forwards the same
  runtime-owned recovered and faulted supervision truth without server-local
  restart policy
- `signal-supervisor-tools` now exposes the machine-readable
  `signal.runtime.device-supervision-boundary` descriptor
- `effigy acceptance:device-supervision-boundary` keeps the shared proof spine
  runnable

This closes `g06.014` intentionally: the supervision substrate is now
contracted, materialized, and proven strongly enough that later clock drift,
duplex mismatch, and endpoint-topology work can build on it instead of
reopening hardware fault ownership.

## Next Task

Continue `g06.015` with Batch 15.1 by freezing the runtime-owned clock-domain
drift, duplex mismatch, discontinuity, and endpoint-topology contract on top of
the closed supervision boundary.
