# 014 Plugin Isolation Policy, Transport Rebind, And Shared-Sandbox Continuity Contract

Status: complete
Owner: core-product
Updated: 2026-03-14
Related contracts: `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`, `docs/contracts/007-plugin-backend-and-host-neutral-delegation-contract.md`, `docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md`, `docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md`, `docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared plugin placement, transport-rebind, and shared-sandbox
continuity contract for `g06.003` so later runtime policy evaluation, CLAP or
VST3 or AU depth, and multi-instance recovery proofs all extend one
runtime-owned meaning instead of drifting into host-local sandbox heuristics.

## Authority hierarchy

Plugin placement and shared-sandbox continuity has one authority chain:

1. `signal-plugin` owns the format-neutral sandbox and lifecycle vocabulary:
   - `PluginFormat`
   - `PluginSandboxCapabilities`
   - `SandboxTransport`
   - `PluginLifecycleState`
   - `PluginReadiness`
   - `PluginFaultKind`
   - `SandboxStateMachine`
2. `signal-runtime` owns the runtime-side placement, grouping, and recovery
   meaning built on top of that vocabulary:
   - plugin sandbox and lifecycle observation
   - plugin chain and execution-topology export
   - interruption, degradation, and rebind semantics
   - shared-boundary blast-radius and continuity interpretation
3. host and adapter crates may realize sandbox processes, transport attach or
   detach, or restart work, but they must not become the authority for:
   - deciding which instances share one authoritative sandbox boundary
   - deciding whether a shared boundary is resumable, restartable, or terminal
   - reconstructing placement or rebind outcomes from private host state when
     runtime-owned snapshots already answer the question

If a consumer cannot explain placement or shared-sandbox continuity through
Signal-owned snapshots, receipts, or export surfaces, it is not yet part of
the shared contract.

## Shared terms

This contract freezes eight shared terms.

### Placement rule

A placement rule is a reusable runtime-owned predicate that selects plugin
instances or plugin types for one isolation outcome without binding that logic
to one product's private allowlist format.

Placement rules may later match on:

- plugin format
- vendor or identity
- declared capability
- reusable safety or verification labels
- additive future shared filters

### Placement policy

A placement policy is the ordered runtime-owned set of placement rules plus the
default fallback outcome.

Products may choose or assemble policy inputs, but the resulting placement
interpretation must stay runtime-owned.

### Sandbox grouping key

A sandbox grouping key is the runtime-owned identity that says which plugin
instances are intended to share one sandbox boundary.

Grouping keys may later be derived from:

- explicit isolated placement
- shared placement by plugin identity or vendor
- grouping by plugin format
- later additive grouping presets

Consumers must not infer grouping by comparing process IDs, transport handles,
or adapter-private routing artifacts.

### Isolation outcome

An isolation outcome is the runtime-owned placement result for one plugin
instance or group.

The first shared outcome vocabulary is:

- `in-process`
- `shared-sandbox`
- `isolated-sandbox`

Later milestones may add stricter policy detail, but they must preserve this
meaning.

### Shared sandbox boundary

A shared sandbox boundary is the authoritative runtime-owned execution boundary
that may contain several plugin instances under one sandbox lifecycle and one
transport attachment path.

The shared boundary is the unit that can degrade, restart, rebind, or fail
terminally as one blast radius even when several plugin instances are members.

### Rebind

Rebind means runtime re-establishes sandbox or transport attachment for an
existing authoritative plugin boundary without pushing continuity ownership
into a product host.

Rebind is a repair action, not a second placement system. It composes with
contract `012`:

- some rebind paths are `Resumable`
- some are `Restartable`
- terminal rebind failure is `Terminal`

### Shared-boundary degradation

Shared-boundary degradation means one sandbox boundary is currently operating
under fault, detach, quarantine, or restart pressure that may affect all
member plugin instances.

The degraded boundary remains runtime-owned even when the member instances have
different immediate readiness or chain-level impact.

### Terminal sandbox boundary

Terminal sandbox boundary means runtime can no longer safely continue or rebind
the current authoritative shared boundary.

Terminal boundary must be exported explicitly rather than inferred from missing
plugin output, missing transports, or a host-private watchdog note.

## Continuity rules

This contract freezes six shared rules.

### Rule 1: placement evaluation stays runtime-owned

Products may supply intent, but runtime owns the final placement outcome and
grouping interpretation.

Host-local policy tables, product browser metadata, or adapter-private process
maps must not become the shared authority.

### Rule 2: grouping is stronger than per-instance heuristics

If several instances share one sandbox boundary, runtime must expose that
shared continuity meaning directly instead of forcing consumers to infer it by
matching instance faults or transport records after the fact.

### Rule 3: rebind semantics compose with interruption taxonomy

Contract `012` remains the top-level interruption vocabulary:

- shared-boundary rebind may be `Resumable`
- shared-boundary restart may be `Restartable`
- shared-boundary failure may be `Terminal`
- some shared-boundary repair paths are additionally `Rebindable`

This milestone does not create a competing plugin-specific recovery language.

### Rule 4: transport continuity is part of the sandbox boundary

Sandbox transport attach, detach, detach-fault, and restart state belong to
the same runtime-owned boundary as plugin lifecycle state. Consumers should not
need separate host-local transport ledgers to know whether a boundary is still
continuing truthfully.

### Rule 5: blast radius is shared evidence

When one shared boundary degrades or fails, all affected instances must remain
explainable through one shared runtime-owned story:

- which boundary degraded
- which placement outcome produced that boundary
- whether continuity is resumable, restartable, or terminal
- whether the blast radius is limited to one boundary or reaches several
  chains or nodes

### Rule 6: host edges expose continuity truth, not recovery policy

Stable host edges may expose placement, sandbox, chain, and supervisor export
state, but they must not reinterpret rebindability or terminal outcome from
product-private repair logic.

## Multi-instance continuity semantics

This contract freezes the first shared multi-instance semantics.

### One boundary may back several instances

Several plugin instances may share one sandbox boundary while still appearing
as distinct node bindings or chain stages.

Runtime-owned grouping meaning must explain that relationship directly.

### Boundary state precedes member interpretation

Consumers should read shared-boundary lifecycle and transport state before
inventing per-instance recovery stories. Member instances may diverge in
readiness or chain impact, but they still inherit one shared boundary outcome.

### Rebind preserves authoritative ownership

If a shared boundary is rebindable, the authoritative runtime-owned plugin
surface survives while runtime repairs transport or sandbox lifecycle.

Products must not create a new sandbox ownership story just because attach or
detach progress is visible.

### Terminal failure is explicit and shared

If one shared boundary fails terminally, all affected member instances should
be explainable through the same terminal boundary outcome even if later product
behavior chooses different UX or replacement actions.

## Current runtime mapping

The current repo baseline already contains the raw runtime-owned surfaces that
this contract builds on.

### Format-neutral substrate

`signal-plugin` already exposes the current reusable sandbox vocabulary:

- `PluginSandboxCapabilities`
- `SandboxTransport`
- `PluginLifecycleState`
- `PluginReadiness`
- `PluginFaultKind`
- `SandboxStateMachine`

### Runtime lifecycle and sandbox observation

`signal-runtime` already exports the current plugin continuity baseline:

- `RuntimePluginSandboxSnapshot`
- `RuntimePluginLifecycleSnapshot`
- `RuntimePluginChainStageSnapshot`
- `RuntimePluginExecutionChainSummary`
- `RuntimePluginChainSnapshot`
- `PluginSandboxLifecycleStage`
- `PluginSandboxTransportStage`

These types now also carry the first runtime-owned placement and continuity
receipts:

- `RuntimePluginPlacementPolicy`
- `RuntimePluginIsolationOutcome`
- placement rule identity and sandbox grouping keys
- shared-boundary member counts
- per-boundary and per-stage interruption-aligned continuity class
- explicit rebindability on shared-sandbox receipts

Batch 3.2 now grounds the contract in real runtime-owned receipt fields rather
than prose alone.

Batch 3.3 now also proves those receipts through focused runtime, public
runtime, shared host-edge, and machine-readable descriptor paths instead of
leaving the contract closure at DTO shape only.

### Interruption and degradation context

The adjacent runtime-owned continuity surfaces are:

- `RuntimeInterruptionSummary`
- `RuntimeFaultStatusSnapshot`
- `RuntimeDegradationSummary`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- `RuntimeSupervisorApi::supervisor_report()`

These surfaces are the canonical consumer path for understanding whether a
plugin boundary is active, rebindable, degraded, or terminal.

## Consumer promises

This contract keeps four promises.

### Products observe one placement truth

Consumers may inspect placement and shared-boundary continuity, but they should
not need host-private grouping or process heuristics to know what runtime
decided.

### Shared-sandbox blast radius stays explicit

If several instances share one boundary, consumers should be able to understand
that blast radius through runtime-owned surfaces rather than a product-local
recovery ledger.

### Future adapter breadth extends the same contract

Later CLAP, VST3, and AU depth may widen capability or transport detail, but
they must reuse this placement and continuity meaning.

### Future milestones refine DTOs, not semantics

Batch 3.2 and later milestones may add new typed policy receipts, grouping
records, or rebind snapshots, but they must preserve the meanings frozen here.

## Deferred scope

This completed contract intentionally defers:

- explicit shared-boundary blast-radius export on runtime snapshots
- deeper dedicated blast-radius DTOs beyond the current lifecycle and chain
  receipts
- backend-specific transport tuning, watchdog policy, or process model detail
- deeper in-process parity proofs beyond the current sandbox-first exercised
  path
- product-local plugin browser, trust UX, or sandbox preset workflow

Those areas belong to later `g06` milestones, but they should now build on
this shared vocabulary rather than replacing it.

## Next Task

Continue `g06.004` with Batch 4.1 by freezing the offline-render recovery and
resumability contract on top of the shared interruption vocabulary before
later runtime session-depth work widens render checkpoint and artifact truth.
