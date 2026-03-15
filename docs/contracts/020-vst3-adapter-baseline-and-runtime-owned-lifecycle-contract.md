# 020 VST3 Adapter Baseline And Runtime-Owned Lifecycle Contract

Status: complete
Owner: core-product
Updated: 2026-03-15
Related contracts: `docs/contracts/008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`, `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`, `docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`, `docs/architecture/package-map.md`

## Purpose

Freeze the first VST3-specific contract alignment for `g06.009` so later
runtime adapter realization can widen real VST3 support without reopening
host-local ownership, format-specific lifecycle semantics, or CLAP-first
assumptions inside shared Signal-owned receipts.

## Authority hierarchy

VST3 support has one authority chain:

1. `signal-plugin` owns the format-neutral plugin vocabulary and shared meaning
   for:
   - plugin identity, feature class, and default I/O shape
   - state, processing, lifecycle, readiness, and fault contracts
   - sandbox capability, transport, and placement substrate
2. `signal-runtime` owns the runtime-side interpretation of VST3 plugins
   through the existing shared contract family:
   - discovery snapshots and scan receipts
   - lifecycle, chain, recall, compensation, and continuity surfaces
   - interruption, fault, and rebind classification
   - report and supervisor export delivery
3. `signal-plugin-vst3` is the intended adapter-owned realization layer for
   VST3-specific detail such as:
   - module or bundle traversal
   - class-factory enumeration and class-category filtering
   - component or controller pairing
   - event and process-context packet mapping
   - backend-native activation, suspend, and teardown helpers
4. host crates may request scans, load plugins, or broker sandbox transport,
   but they must not become the authority for:
   - VST3 capability meaning
   - VST3 lifecycle or continuity classification
   - Linux-hosted VST3 support claims
   - consumer-visible discovery or fault conclusions when shared runtime-owned
     receipts already answer the question

If a VST3 conclusion cannot be explained through Signal-owned plugin and
runtime receipts, it is not yet part of the shared contract.

## Shared mapping rules

This contract freezes seven shared VST3 mapping rules.

### Rule 1: backend identity remains one shared tag

`PluginFormat::Vst3` remains the canonical backend identity tag.

VST3-specific identifiers such as class ids, module paths, or factory
categories may help the adapter realize discovery, but consumer-visible backend
meaning must still flow through shared Signal-owned plugin identity and
runtime-owned discovery receipts rather than a separate VST3-native taxonomy.

### Rule 2: VST3 class and category detail must collapse into shared discovery meaning

VST3 discovery may require adapter-private enumeration of classes, subcategories,
controller relationships, and module contents, but the shared consumer answer
still belongs to Signal-owned surfaces such as:

- `RuntimePluginDiscoverySnapshot`
- `RuntimePluginScanReceipt`
- `RuntimePluginDiscoveredTypeRecord`
- `RuntimePluginFormatCoverageRecord`
- `RuntimePluginCapabilityCoverageSummary`

If VST3 needs richer consumer-visible discovery detail later, that detail must
be promoted into shared Signal-owned DTOs additively before consumers depend on
it.

### Rule 3: component/controller split is adapter-private realization, not shared lifecycle meaning

VST3 may realize one logical plugin through several backend-native objects such
as a processor and controller pair. That split does not create a second shared
lifecycle model.

Shared lifecycle meaning still belongs to the existing Signal-owned plugin
contract family:

- readiness
- lifecycle state
- recall state
- isolation outcome
- interruption class
- restartable, resumable, rebindable, or terminal continuity

Pairing failure or mismatch may later become additive diagnostic evidence, but
it must not become a host-local lifecycle taxonomy.

### Rule 4: bus and event topology map into format-neutral capability surfaces first

VST3 audio buses, event buses, and default arrangement detail must map through
the existing format-neutral capability seam first:

- feature class
- audio and event I/O shape
- processing readiness
- parameter or state capability

Backend-native speaker-layout names, unit topology, and controller-side editing
structure remain adapter-private until Signal promotes them explicitly in later
cross-adapter capability work.

### Rule 5: state and recall stay runtime-owned even if VST3 state realization is split

VST3 may require adapter-private handling for processor or controller state,
program lists, or host context, but the shared continuity and recall answer
still belongs to Signal-owned surfaces:

- recall payload and handoff snapshots
- lifecycle and chain snapshots
- offline delegated execution boundaries
- fault and interruption receipts

Products and hosts must not reconstruct one VST3-specific recall or recovery
story from adapter internals when runtime-owned receipts already provide the
authoritative state.

### Rule 6: Linux-hosted VST3 breadth must be explicit, not implied

`g06.009` freezes Linux-hosted VST3 support as an explicit requirement for the
later adapter baseline.

That means Batch 9.2 must make Linux scan and load coverage explicit through
Signal-owned crates and receipts. The current package-map intent or legacy C++
code is not enough on its own, and support must not be implied just because the
format tag exists.

### Rule 7: legacy VST3 code is reference material, not contract authority

The current legacy C++ VST3 registry and backend code may inform implementation
strategy, but it does not define the shared Rust runtime contract.

The shared authority remains:

- `signal-plugin`
- `signal-runtime`
- future `signal-plugin-vst3`
- supervisor and stable host-edge export of those Signal-owned surfaces

## Current repo mapping

The repo already contains the substrate this contract builds on:

- `PluginFormat::Vst3` is already part of the shared plugin identity surface
- runtime discovery, lifecycle, continuity, and export proofs already include
  VST3-tagged plugin identities in shared DTOs
- plugin placement, sandbox grouping, interruption, and continuity semantics
  are already format-neutral and therefore apply to VST3 without a second
  lifecycle contract
- `docs/architecture/package-map.md` already reserves `signal-plugin-vst3` as
  the intended Trust-Edge adapter crate
- legacy C++ VST3 registry and backend code already exist, but only as
  implementation reference material rather than shared Rust contract authority

Batch 9.1 freezes how those pieces should line up before the real adapter
baseline lands.

## Explicit contract gaps before runtime realization

Batch 9.1 intentionally records the gaps that still exist:

- no Rust `signal-plugin-vst3` adapter crate is currently realizing the shared
  contract
- no runtime-owned VST3 discovery path yet exports module, class, and pair
  resolution through real adapter-backed receipts
- no explicit Linux-hosted VST3 scan or load path is yet proven through
  runtime-owned surfaces
- no additive shared DTO yet captures VST3-specific controller or processor
  mismatch evidence
- VST3 unit, program-list, note-expression, and richer event-model depth remain
  deferred to later `g06.011` or `g06.012` parity work unless Batch 9.2 needs a
  minimal additive promotion

These are runtime realization gaps, not permission to bypass the shared
contract.

## Batch 9.1 outcome

Batch 9.1 freezes the first VST3 adapter alignment boundary:

- VST3-specific realization detail is now explicitly mapped onto the existing
  backend-neutral capability and continuity contract family
- Linux-hosted VST3 breadth is now an explicit runtime-owned requirement rather
  than package-map intent
- component/controller and module/class detail remain adapter-private
  realization until Signal promotes additive shared DTOs
- later runtime adapter work now has one contract to extend without reopening
  host-local lifecycle or discovery ownership

## Batch 9.2 outcome

Batch 9.2 turns the contract into the first real runtime-owned VST3 baseline:

- `signal-plugin-vst3` now exists as a real Rust adapter crate rather than
  package-map intent only
- the adapter now owns bounded VST3 realization detail for:
  - platform-specific scan-root presets, including explicit Linux roots
  - class/controller pairing
  - descriptor and capability projection
  - bounded shared-memory session planning
- `signal-host-local` and `signal-host-server` now both feed VST3 discovery
  results into runtime-owned discovery receipts rather than host-local catalogs
- VST3 sandbox ensure on both hosts now records runtime-owned lifecycle,
  instance-state, and transport-attachment receipts without creating a
  VST3-only lifecycle taxonomy
- Linux-hosted VST3 coverage is now explicit through the server-host proof path
  instead of package-map implication alone

This still defers consumer-facing conformance proof, richer VST3 event or unit
depth, and broader AU/cross-adapter parity work to later batches.

## Batch 9.3 outcome

Batch 9.3 turns the VST3 baseline into a shared consumer boundary:

- `signal-runtime` now has a downstream-style public proof that VST3 discovery
  and lifecycle truth remain consumable through shared runtime receipts
- both stable host edges now prove they forward the same VST3 discovery and
  lifecycle state through `RuntimeSupervisorReport` without adapter-local
  reconstruction
- `signal-supervisor-tools` now exposes the machine-readable
  `signal.runtime.vst3-boundary` descriptor, and the repo-owned
  `effigy acceptance:vst3-boundary` task keeps that proof runnable
- VST3-specific realization now has one closed bounded consumer seam without
  widening shared capability meaning beyond the existing backend-neutral plugin
  contract

Deferred scope remains explicit: richer VST3 event, unit, and program-list
depth still belong to later cross-adapter or parity work rather than this first
baseline.

## Next Task

Continue `g06.010` with Batch 10.1 by mapping AU-specific discovery,
lifecycle, and macOS-scoped capability detail onto the shared backend-neutral
plugin contract before runtime-owned AU realization widens.
