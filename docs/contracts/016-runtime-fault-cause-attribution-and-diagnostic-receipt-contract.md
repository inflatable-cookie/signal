# 016 Runtime Fault-Cause Attribution And Diagnostic Receipt Contract

Status: complete
Owner: core-product
Updated: 2026-03-14
Related contracts: `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`, `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`, `docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md`, `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`, `docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md`, `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`, `docs/contracts/015-offline-render-recovery-and-resumability-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared fault-cause attribution and diagnostic receipt contract
for `g06.005` so later profiling, soak, hardware, plugin, and recovery work
can point at one runtime-owned causal story instead of forcing products to
infer cause from mixed counters, host-private notes, or watchdog prose.

## Authority hierarchy

Runtime fault attribution has one authority chain:

1. `signal-runtime` owns the canonical causal meaning:
   - broad fault and recovery posture
   - interruption class and rebindability
   - runtime xrun and degraded-service evidence
   - plugin, transport, broker, and sandbox failure evidence
   - deferred-work and prework pressure evidence
2. host and adapter crates may supply adjacent callback, backend, and device
   observations, but they must not become the authority for deciding:
   - the canonical primary cause for one runtime boundary
   - whether evidence should be treated as advisory or contract-grade
   - whether a fault is resumable, restartable, recoverable, or terminal
3. supervisor/export surfaces and stable host edges may expose the causal
   meaning, but they must not reinterpret it:
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - `RuntimeFaultStatusSnapshot`
   - `RuntimeInterruptionSummary`
   - `RuntimeDegradationSummary`
   - `RuntimePerformanceSnapshot`
   - `RuntimePerformanceTraceReceipt`
   - stable host-edge `supervisor_report()` surfaces

If a consumer cannot explain a fault through Signal-owned snapshots, receipts,
or exported evidence, it is not yet part of the shared diagnostic contract.

## Shared terms

This contract freezes seven shared terms.

### Causal receipt

A causal receipt is a runtime-owned explanation of why runtime entered or
remained in a degraded, recovering, or faulted posture.

Causal receipts are stronger than counters alone: they say which fault family
is primary, which evidence contributed, and how the cause composes with the
interruption taxonomy from contract `012`.

### Primary cause

Primary cause is the single runtime-owned cause family that best explains the
current active fault boundary.

Only one primary cause may be canonical for a given capture. Other evidence may
still contribute, but downstream consumers should not need to invent precedence
rules themselves.

### Contributing evidence

Contributing evidence is supporting runtime-owned or host-adjacent detail that
helps explain the primary cause without replacing it.

Examples include:

- host callback overrun counts during xrun pressure
- plugin transport fault records alongside sandbox restart cycles
- deferred-work throttle or starvation counters during safe-mode pressure

### Advisory host evidence

Advisory host evidence is host or backend detail that may sharpen diagnosis but
does not outrank runtime-owned causal meaning.

Examples already present in current surfaces include:

- host callback count
- host callback overrun count
- host backend xrun count
- host device loss count
- host restart attempt and failure counts

### Pressure fault

A pressure fault is a capacity or scheduling loss that degrades progress
without necessarily implying attachment loss or a terminal boundary.

The first shared pressure families are:

- `xrun pressure`
- `callback pressure`
- `deferred-work pressure`

### Boundary fault

A boundary fault is a failure on a plugin, device, or transport-attached
execution boundary where repair, restart, or rebind semantics matter as much as
raw error counts.

The first shared boundary families are:

- `plugin boundary fault`
- `device path fault`

### Causal family

A causal family is the reusable shared category that later DTOs should expose
directly instead of forcing products to infer meaning from unrelated counters.

This contract freezes five first-family categories:

- `xrun pressure`
- `callback pressure`
- `plugin boundary fault`
- `device path fault`
- `deferred-work pressure`

## First causal families

This milestone freezes the meaning of the first five shared fault families.

### Xrun pressure

`Xrun pressure` means runtime-owned execution fell behind realtime budget as
shown by runtime xrun evidence, xrun-overload posture, or adjacent runtime
performance receipts.

This family is runtime-owned first. Host backend xrun counters may contribute,
but they do not replace runtime xrun meaning.

### Callback pressure

`Callback pressure` means host callback cadence or callback budget loss is
visible at the host edge and is relevant to runtime diagnosis.

This family is advisory until runtime promotes it into canonical causal
meaning. Host callback and callback-overrun counts are valid evidence, but they
must not create a competing host-local recovery taxonomy.

### Plugin boundary fault

`Plugin boundary fault` means plugin sandbox, lifecycle, transport, or binding
failure is the primary reason one runtime boundary is degraded, recovering, or
faulted.

This family builds directly on:

- plugin lifecycle and sandbox continuity receipts
- transport fault summaries
- sandbox operation and broker failure events
- missing plugin binding evidence

### Device path fault

`Device path fault` means device loss, device restart pressure, or device
recovery failure is the primary reason one runtime boundary is interrupted.

This family composes naturally with `Restartable`, `Recoverable`, `Terminal`,
and `Rebindable` from contract `012`.

### Deferred-work pressure

`Deferred-work pressure` means non-realtime work or prework service pressure is
causing defer, throttle, starvation, yield, or readiness-adjacent degradation
that matters to shared diagnostics.

This family is runtime-owned only when explained through runtime service,
forecast, prework, or deferred-work receipts. Product-local background task
queues or UI batching are out of scope.

## Mapping rules

This contract freezes six shared mapping rules.

### Rule 1: one canonical cause outranks raw evidence

Products may inspect many counters and records, but they should not need to
decide precedence among xrun, plugin, device, or deferred-work evidence.
Runtime-owned causal receipts must do that first.

### Rule 2: interruption meaning stays downstream of causal meaning

Contract `012` remains the top-level interruption vocabulary. Causal receipts
must map into it instead of creating a competing diagnostic taxonomy.

### Rule 3: host callback evidence is additive, not sovereign

Host callback cadence, backend xruns, and callback overruns may sharpen causal
explanation, but they do not outrank runtime-owned readiness, interruption, or
recovery classification.

### Rule 4: plugin and device causes carry boundary semantics

Plugin and device causes must stay explainable through restart, rebind, shared
boundary blast radius, and terminal outcome, not just through rising counter
values.

### Rule 5: deferred-work pressure must point at runtime service policy

Deferred-work pressure is only shared-contract evidence when it can be explained
through runtime-owned prework or deferred-service surfaces such as starvation,
throttle, defer, or gate state.

### Rule 6: export and host edges expose cause, not diagnosis prose

Supervisor export and stable host edges may summarize causal receipts, but they
must not force consumers to reconstruct cause from logs, crash notes, or
host-private helper logic.

## Interruption and recovery mapping

This Batch 5.1 contract freezes the first shared cause-to-posture mapping.

### Xrun pressure

- usually maps to `Recoverable` or `Resumable` posture first
- may contribute to `Restartable` posture when safe-mode or watchdog repair is
  active
- does not alone imply `Terminal` unless runtime later faults for a stronger
  canonical cause

### Callback pressure

- usually remains advisory while runtime stays `Steady`
- may contribute to `Resumable` or `Recoverable` posture when runtime yields,
  throttles, or begins xrun recovery
- should not become `Terminal` without stronger runtime-owned boundary failure

### Plugin boundary fault

- commonly maps to `Restartable`
- may also be `Rebindable`
- becomes `Terminal` when the authoritative plugin boundary can no longer be
  safely repaired

### Device path fault

- commonly maps to `Restartable`
- may also be `Rebindable` during device or stream repair
- becomes `Terminal` when runtime cannot safely continue the current boundary

### Deferred-work pressure

- commonly maps to `Resumable` when runtime defers or throttles work
- may map to `Recoverable` when safe mode, recovery overlap, or transport or
  plugin gates are already active
- does not alone imply `Terminal`

## Current runtime mapping

The current repo baseline already contains the broad runtime-owned surfaces that
this contract builds on.

### Broad fault and interruption posture

`signal-runtime` already owns the current posture seams:

- `RuntimeFaultStatusSnapshot`
- `RuntimeInterruptionSummary`
- `RuntimeDegradationSummary`

These types already answer:

- current recovery posture
- broad primary fault cause
- active-fault counts
- xrun, plugin, transport, watchdog, and device-loss activity
- whether interruption is active, resumable, restartable, recoverable, or
  terminal

### Runtime performance and pressure evidence

`signal-runtime` already owns the first runtime pressure evidence:

- `RuntimeEngineBlockSnapshot`
- `RuntimePerformanceSnapshot`
- `RuntimePerformanceTraceReceipt`
- `RuntimeDeferredServiceReceipt`

These types already expose:

- runtime xrun counts
- prework starvation, throttle, and yield counters
- background-service defer, throttle, and abort decisions
- plugin and transport gate activity
- queue depth and backlog pressure

### Plugin and transport fault evidence

`signal-runtime` already owns the current plugin and transport evidence:

- `RuntimePluginLifecycleSnapshot`
- `RuntimePluginChainSnapshot`
- `RuntimeObservationDiagnostics`
- `TransportFaultSummary`
- `TransportSessionSummary`
- broker, sandbox-operation, and transport-fault records

These are the current evidence seams for plugin-boundary fault attribution.

### Host-adjacent callback and device evidence

Current performance and host-report surfaces already expose advisory host
evidence:

- host callback counts and callback interval
- host backend xrun count
- host callback overrun count
- host device loss count
- host restart attempt and failure counts

This contract keeps that evidence explicitly subordinate to runtime-owned cause
meaning.

## Deferred scope

This Batch 5.1 contract intentionally defers:

- per-event stack traces, heap dumps, or fleet telemetry
- product-specific diagnostic UX, alert routing, or session journaling
- durable remote diagnostics pipelines
- final DTO shape for richer causal receipts and summaries
- promotion of advisory host callback evidence into stronger mandatory runtime
  cause selection before Batch 5.2 proves the receipt family is worth freezing

## Current baseline surfaces

The current repo-owned baseline that this contract builds on is:

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
- `RuntimeFaultStatusSnapshot`
- `RuntimeInterruptionSummary`
- `RuntimeDegradationSummary`
- `RuntimeDiagnosticsSnapshot`
- `RuntimeEngineBlockSnapshot`
- `RuntimePerformanceSnapshot`
- `RuntimePerformanceTraceReceipt`
- `RuntimeObservationDiagnostics`
- `TransportFaultSummary`
- `TransportSessionSummary`
- `RuntimeDeferredServiceReceipt`
- `RuntimeSupervisorReport`
- `RuntimeObservationReport`
- `RuntimeSupervisorApi::supervisor_report()`
- `docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md`
- `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`
- `docs/contracts/015-offline-render-recovery-and-resumability-contract.md`

## Batch 5.2 landed receipt family

Batch 5.2 moves this contract beyond vocabulary and into typed runtime-owned
receipt surfaces.

The first landed DTO family is:

- `RuntimeFaultDiagnosticReceipt`
- `RuntimeFaultContributionReceipt`
- `RuntimeFaultDiagnosticFamily`
- `RuntimeFaultDiagnosticAuthority`

The first landed runtime-owned export surfaces are:

- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- `RuntimeProfilingReceipt`
- stable host-edge `supervisor_report()` consumers through the already-frozen
  host boundary

The landed mapping keeps three rules explicit:

1. `primary_family` stays canonical and runtime-owned. It is derived from
   runtime posture, recovery state, interruption class, and current fault
   boundary rather than from host callback heuristics.
2. `contributions` may include both runtime-canonical and host-advisory
   evidence. Host callback and backend counters remain additive evidence only.
3. deferred-work pressure may be the canonical family when runtime-owned defer,
   throttle, starvation, or yield surfaces are the best explanation, even when
   historical host or plugin counters are also present.

This means downstream consumers can now read one typed receipt for:

- canonical cause family
- underlying broad fault cause
- interruption class and recovery state
- safe-mode and rebindability posture
- contributing runtime and host-adjacent evidence

## Batch 5.3 consumer-facing proof boundary

Batch 5.3 closes this contract with one explicit consumer-facing inspection
seam:

- `signal-supervisor-tools --describe-fault-diagnostic-boundary`
- `effigy acceptance:fault-diagnostic-boundary --repo .`

That seam proves:

- a downstream-style runtime consumer can read canonical primary-family and
  contribution evidence from public runtime surfaces
- stable local and server host edges forward the same receipt through
  `supervisor_report()`
- consumers can inspect the shared fault-diagnostic boundary without private
  host helpers or log parsing

## Next Task

Continue `g06.006` with Batch 6.1 by defining the first per-block timing and
pressure snapshot contract so later instrumentation work can compose with the
now-closed fault-diagnostic boundary instead of inventing a second profiling
taxonomy.
