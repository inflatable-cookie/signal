# 021 AU Adapter Baseline And Runtime-Owned Lifecycle Contract

Status: complete
Owner: core-product
Updated: 2026-03-15
Related contracts: `docs/contracts/008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`, `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`, `docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`, `docs/architecture/package-map.md`

## Purpose

Freeze the first AU-specific contract alignment for `g06.010` so later runtime
adapter realization can widen real Audio Unit support without reopening
host-local ownership, macOS-only lifecycle semantics, or product-local wrapper
authority inside shared Signal-owned receipts.

## Authority hierarchy

AU support has one authority chain:

1. `signal-plugin` owns the format-neutral plugin vocabulary and shared meaning
   for:
   - plugin identity, feature class, and default I/O shape
   - state, processing, lifecycle, readiness, and fault contracts
   - sandbox capability, transport, and placement substrate
2. `signal-runtime` owns the runtime-side interpretation of AU plugins through
   the existing shared contract family:
   - discovery snapshots and scan receipts
   - lifecycle, chain, recall, compensation, and continuity surfaces
   - interruption, fault, and rebind classification
   - report and supervisor export delivery
3. `signal-plugin-au` is the intended adapter-owned realization layer for
   AU-specific detail such as:
   - component or unit traversal and subtype or manufacturer filtering
   - AudioComponent discovery and factory binding
   - AU instance bring-up, initialization, and teardown helpers
   - parameter-tree, bus-layout, and render-context bridging
   - macOS-hosted capability and host-context glue
4. host crates may request scans, load plugins, or broker sandbox transport,
   but they must not become the authority for:
   - AU capability meaning
   - AU lifecycle or continuity classification
   - consumer-visible discovery or failure conclusions when shared runtime-owned
     receipts already answer the question
   - macOS-scoped AU support claims beyond what Signal-owned crates actually
     realize and prove

If an AU conclusion cannot be explained through Signal-owned plugin and runtime
receipts, it is not yet part of the shared contract.

## Shared mapping rules

This contract freezes seven shared AU mapping rules.

### Rule 1: backend identity remains one shared tag

`PluginFormat::Au` remains the canonical backend identity tag.

AU-specific identifiers such as AudioComponent type, subtype, manufacturer, or
bundle metadata may help the adapter realize discovery, but consumer-visible
backend meaning must still flow through shared Signal-owned plugin identity and
runtime-owned discovery receipts rather than a separate AU-native taxonomy.

### Rule 2: AudioComponent and bundle traversal detail must collapse into shared discovery meaning

AU discovery may require adapter-private enumeration of components, bundles,
component descriptions, versions, and subtype or manufacturer filters, but the
shared consumer answer still belongs to Signal-owned surfaces such as:

- `RuntimePluginDiscoverySnapshot`
- `RuntimePluginScanReceipt`
- `RuntimePluginDiscoveredTypeRecord`
- `RuntimePluginFormatCoverageRecord`
- `RuntimePluginCapabilityCoverageSummary`

If AU needs richer consumer-visible discovery detail later, that detail must be
promoted into shared Signal-owned DTOs additively before consumers depend on
it.

### Rule 3: AU instance bring-up remains one shared lifecycle, not a host-local wrapper model

AU may realize one logical plugin through backend-native component discovery,
instantiation, property negotiation, render callbacks, and teardown. That does
not create a second shared lifecycle model.

Shared lifecycle meaning still belongs to the existing Signal-owned plugin
contract family:

- readiness
- lifecycle state
- recall state
- isolation outcome
- interruption class
- restartable, resumable, rebindable, or terminal continuity

Instance initialization or property negotiation failures may later become
additive diagnostic evidence, but they must not become a host-local AU-only
lifecycle taxonomy.

### Rule 4: AU buses, events, and parameter trees map into format-neutral capability surfaces first

AU audio buses, MIDI or event handling, parameter trees, and default channel
layout detail must map through the existing format-neutral capability seam
first:

- feature class
- audio and event I/O shape
- processing readiness
- parameter or state capability

AU-specific tree structure, Cocoa view or editor detail, and deeper host UI
integration remain adapter-private until Signal promotes them explicitly in
later cross-adapter capability work.

### Rule 5: state and recall stay runtime-owned even if AU realization is property-heavy

AU may require adapter-private handling for property snapshots, state blobs,
preset references, or host context, but the shared continuity and recall answer
still belongs to Signal-owned surfaces:

- recall payload and handoff snapshots
- lifecycle and chain snapshots
- offline delegated execution boundaries
- fault and interruption receipts

Products and hosts must not reconstruct one AU-specific recall or recovery
story from adapter internals when runtime-owned receipts already provide the
authoritative state.

### Rule 6: AU support remains explicitly macOS-scoped unless Signal widens it later

`g06.010` freezes AU support as a macOS-scoped requirement for the later
adapter baseline.

That means Batch 10.2 must make macOS scan and load coverage explicit through
Signal-owned crates and receipts. AU support must not be implied on Linux or
Windows just because the format tag exists in shared plugin identity.

### Rule 7: legacy AU or host-local wrapper code is reference material, not contract authority

Any existing product-local or legacy AU wrapper code may inform implementation
strategy, but it does not define the shared Rust runtime contract.

The shared authority remains:

- `signal-plugin`
- `signal-runtime`
- future `signal-plugin-au`
- supervisor and stable host-edge export of those Signal-owned surfaces

## Current repo mapping

The repo already contains the substrate this contract builds on:

- `PluginFormat::Au` already exists as part of the shared plugin identity
  surface
- runtime discovery, lifecycle, continuity, and export proofs already rely on
  backend-neutral plugin DTOs that can carry AU identities without a second
  lifecycle contract
- plugin placement, sandbox grouping, interruption, and continuity semantics
  are already format-neutral and therefore apply to AU without an AU-only
  recovery boundary
- `docs/architecture/package-map.md` already reserves `signal-plugin-au` as the
  intended Trust-Edge adapter crate
- `g06.009` now closes the first real non-CLAP adapter baseline, so AU can
  align against the same runtime-owned discovery and lifecycle seams rather
  than reopening contract shape

Batch 10.1 freezes how those pieces should line up before the real adapter
baseline lands.

## Explicit contract gaps before runtime realization

Batch 10.1 intentionally records the gaps that still exist:

- no Rust `signal-plugin-au` adapter crate is currently realizing the shared
  contract
- no runtime-owned AU discovery path yet exports AudioComponent traversal or
  macOS-scoped scan results through real adapter-backed receipts
- no explicit AU sandbox or instance bring-up path is yet proven through
  runtime-owned lifecycle, instance-state, and transport receipts
- no additive shared DTO yet captures AU-specific instance-initialization,
  property, or render-context mismatch evidence
- AU parameter-tree depth, preset documents, editor or Cocoa view integration,
  and richer MIDI or event-model breadth remain deferred to later
  cross-adapter or product-facing work unless Batch 10.2 needs a minimal
  additive promotion

These are runtime realization gaps, not permission to bypass the shared
contract.

## Batch 10.1 outcome

Batch 10.1 freezes the first AU adapter alignment boundary:

- AU-specific realization detail is now explicitly mapped onto the existing
  backend-neutral capability and continuity contract family
- macOS-scoped AU breadth is now an explicit runtime-owned requirement rather
  than package-map intent
- AudioComponent traversal, property negotiation, and instance bring-up remain
  adapter-private realization until Signal promotes additive shared DTOs
- later runtime adapter work now has one contract to extend without reopening
  host-local lifecycle, discovery, or wrapper ownership

## Batch 10.2 outcome

Batch 10.2 turns the contract into the first real runtime-owned AU baseline:

- `signal-plugin-au` now exists as a real Rust adapter crate rather than
  package-map intent only
- the adapter now owns bounded AU realization detail for:
  - macOS component-root presets
  - AudioComponent-style identity projection
  - descriptor and capability projection
  - bounded shared-memory session planning
- `signal-host-local` and `signal-host-server` now both feed AU discovery
  results into runtime-owned discovery receipts rather than host-local catalogs
- AU sandbox ensure on both hosts now records runtime-owned lifecycle,
  instance-state, and transport-attachment receipts without creating an AU-only
  lifecycle taxonomy
- macOS-scoped AU coverage is now explicit through focused host proof paths
  instead of package-map implication alone

This still defers consumer-facing conformance proof, richer AU parameter-tree
or preset depth, and broader cross-adapter parity work to later batches.

## Batch 10.3 outcome

Batch 10.3 closes the first bounded AU consumer seam:

- AU discovery and lifecycle truth are now proven consumable through public
  `signal-runtime` reexports rather than adapter-local AU helpers
- both stable host edges now prove that `supervisor_report()` forwards the same
  AU discovery and lifecycle receipts without host-private AU reconstruction
- `signal-supervisor-tools` now exposes a machine-readable
  `signal.runtime.au-boundary` descriptor and the repo-owned
  `effigy acceptance:au-boundary --repo .` validation seam
- the AU baseline is now closed as a shared consumer boundary, while richer AU
  parameter-tree, preset, editor, and event-model depth remain explicitly
  deferred to later cross-adapter work

## Next Task

Continue `g06.011` with Batch 11.1 by freezing the backend capability parity,
Linux plugin-support, and cross-adapter conformance contract on top of the now
closed CLAP, VST3, and AU runtime-owned adapter boundaries.
