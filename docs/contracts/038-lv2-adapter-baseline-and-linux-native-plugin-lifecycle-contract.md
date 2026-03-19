# 038 LV2 Adapter Baseline And Linux-Native Plugin Lifecycle Contract

Status: complete
Owner: core-product
Updated: 2026-03-17
Related contracts: `docs/contracts/008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`, `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`, `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first LV2-specific contract alignment for `g07.007` so later
runtime adapter realization can widen real LV2 support without reopening
host-local ownership, Linux-only wrapper semantics, or adapter-private
manifest interpretation inside shared Signal-owned receipts.

## Authority hierarchy

LV2 support has one authority chain:

1. `signal-plugin` owns the format-neutral plugin vocabulary and shared meaning
   for:
   - backend identity and feature class
   - default audio or event I/O shape
   - state, processing, lifecycle, readiness, and fault contracts
   - sandbox capability, transport, and placement substrate
2. `signal-runtime` owns the runtime-side interpretation of LV2 plugins
   through the existing shared contract family:
   - discovery snapshots and scan receipts
   - lifecycle, chain, recall, compensation, and continuity surfaces
   - interruption, fault, and rebind classification
   - report and supervisor export delivery
3. a later Signal-owned LV2 adapter layer is expected to own LV2-specific
   realization detail such as:
   - bundle and manifest traversal
   - URI, class, and feature projection
   - port and atom or event capability mapping
   - Linux-native instantiation, activation, and teardown helpers
   - worker, UI, and extension negotiation that has not yet been promoted into
     shared DTOs
4. host crates may request scans, load plugins, or broker sandbox transport,
   but they must not become the authority for:
   - LV2 capability meaning
   - LV2 lifecycle or continuity classification
   - Linux-native LV2 support claims
   - consumer-visible discovery or failure conclusions when shared
     runtime-owned receipts already answer the question

If an LV2 conclusion cannot be explained through Signal-owned plugin and
runtime receipts, it is not yet part of the shared contract.

## Shared mapping rules

This contract freezes seven shared LV2 mapping rules.

### Rule 1: LV2 must land as a shared backend identity, not a Linux host special case

Batch 7.1 records one explicit requirement: LV2 must be promoted into the
shared backend identity surface rather than treated as host-local Linux glue.

That means later runtime realization must add LV2 through Signal-owned backend
identity and discovery receipts instead of product-local scanners, Linux-only
wrapper tags, or private host catalog entries.

### Rule 2: bundle, manifest, and URI traversal must collapse into shared discovery meaning

LV2 discovery may require adapter-private handling of bundles, manifests, URIs,
classes, and feature declarations, but the shared consumer answer still
belongs to Signal-owned surfaces such as:

- `RuntimePluginDiscoverySnapshot`
- `RuntimePluginScanReceipt`
- `RuntimePluginDiscoveredTypeRecord`
- `RuntimePluginFormatCoverageRecord`
- `RuntimePluginCapabilityCoverageSummary`

If LV2 needs richer consumer-visible discovery detail later, that detail must
be promoted into shared Signal-owned DTOs additively before consumers depend on
it.

### Rule 3: Linux-native plugin bring-up remains one shared lifecycle, not a host-local wrapper model

LV2 may realize one logical plugin through Linux-native bundle loading,
descriptor resolution, feature negotiation, worker hooks, and teardown. That
does not create a second shared lifecycle model.

Shared lifecycle meaning still belongs to the existing Signal-owned plugin
contract family:

- readiness
- lifecycle state
- recall state
- isolation outcome
- interruption class
- restartable, resumable, rebindable, or terminal continuity

Instantiation or feature-negotiation failure may later become additive
diagnostic evidence, but it must not become a host-local LV2-only lifecycle
taxonomy.

### Rule 4: ports, atoms, and event capabilities map into format-neutral capability surfaces first

LV2 audio ports, control ports, atom/event support, and default layout detail
must map through the existing format-neutral capability seam first:

- feature class
- audio and event I/O shape
- processing readiness
- parameter or state capability

LV2 extension-specific worker, UI, atom, patch, time-position, or custom port
metadata remain adapter-private until Signal promotes them explicitly in later
cross-adapter capability work.

### Rule 5: state and recall stay runtime-owned even when LV2 realization is feature-heavy

LV2 may require adapter-private handling for state blobs, preset resources,
URIDs, worker-backed state sync, or host features, but the shared continuity
and recall answer still belongs to Signal-owned surfaces:

- recall payload and handoff snapshots
- lifecycle and chain snapshots
- offline delegated execution boundaries
- fault and interruption receipts

Products and hosts must not reconstruct one LV2-specific recall or recovery
story from adapter internals when runtime-owned receipts already provide the
authoritative state.

### Rule 6: Linux-native LV2 support must be explicit, not implied

`g07.007` freezes LV2 as an explicitly Linux-native plugin breadth milestone.

That means Batch 7.2 must make Linux scan and load coverage explicit through
Signal-owned crates and receipts. LV2 support must not be implied just because
the Linux plugin story exists in roadmap prose.

### Rule 7: extension depth stays deferred until Signal promotes it additively

LV2-specific worker, UI, patch, atom, state-path, and custom extension depth
remain deferred unless later batches promote them into shared DTOs.

The first baseline is discovery, lifecycle, capability, and bounded Linux-native
runtime ownership. It is not a blank check to smuggle extension semantics into
host-private or adapter-private surfaces and call them portable.

## Current repo mapping

The repo already contains the substrate this contract builds on:

- `signal-plugin` already owns the shared backend-neutral plugin vocabulary
- `signal-runtime` already owns discovery, lifecycle, continuity, and export
  seams that later LV2 work must extend rather than replace
- plugin placement, sandbox grouping, interruption, and continuity semantics
  are already format-neutral and therefore apply to LV2 without an LV2-only
  recovery boundary
- `g06.009`, `g06.010`, and `g06.011` already froze and proved the first
  VST3, AU, and cross-adapter breadth seams, so LV2 now has a concrete shared
  contract family to align against instead of inventing one

Batch 7.1 freezes how those pieces should line up before the real adapter
baseline lands.

## Explicit contract gaps before runtime realization

Batch 7.1 intentionally records the gaps that still exist:

- no shared LV2 backend identity has been added to `signal-plugin` yet
- no Rust LV2 adapter crate currently realizes the shared contract
- no runtime-owned LV2 discovery path yet exports bundle, manifest, URI, or
  Linux-native scan results through real adapter-backed receipts
- no explicit LV2 sandbox or instance bring-up path is yet proven through
  runtime-owned lifecycle, instance-state, and transport receipts
- no additive shared DTO yet captures LV2-specific feature-negotiation,
  worker, or extension mismatch evidence
- LV2 worker, UI, atom, patch, preset, and richer event-model depth remain
  deferred to later Linux or cross-adapter work unless Batch 7.2 needs a
  minimal additive promotion

These are runtime realization gaps, not permission to bypass the shared
contract.

## Batch 7.1 outcome

Batch 7.1 freezes the first LV2 adapter alignment boundary:

- LV2-specific discovery, lifecycle, and Linux-native realization detail are
  now explicitly mapped onto the existing backend-neutral capability and
  continuity contract family
- LV2 is now recorded as an explicit shared backend-identity and Linux-native
  runtime requirement rather than a roadmap implication
- manifest, URI, extension, and worker detail remain adapter-private
  realization until Signal promotes additive shared DTOs
- later runtime adapter work now has one contract to extend without reopening
  host-local Linux plugin ownership

## Batch 7.2 outcome

Batch 7.2 realizes the first bounded LV2 adapter baseline:

- `signal-plugin` now exposes LV2 as a shared backend identity instead of a
  contract-only gap
- `signal-plugin-lv2` now owns the first Linux-native LV2 scan-root,
  bundle-path, manifest-path, URI, and bounded capability projection surface
- the server host now feeds that LV2 realization into runtime-owned discovery,
  lifecycle, instance-state, transport, and parity receipts
- Linux-native LV2 support is now explicit on shared runtime receipts rather
  than implied by roadmap prose or private host assumptions

Worker, UI, patch, URID, and richer extension depth remain deferred.

## Batch 7.3 outcome

Batch 7.3 closes the bounded LV2 proof seam:

- public runtime proofs now verify LV2 discovery, lifecycle, transport, and
  Linux-only platform scope through shared runtime reports without private
  adapter access
- the stable server host edge now proves the same LV2 truth on supervisor
  export without Linux-host-local reconstruction
- `signal-supervisor-tools` now exposes the machine-readable
  `signal.runtime.lv2-boundary` descriptor, and Effigy owns a repo-run
  acceptance lane for the LV2 boundary

The first bounded LV2 adapter baseline is now closed.

## Next Task

Continue `g07.008` with Batch 8.2 by aligning lifecycle, render, failure, and
placement receipts across Linux adapters so supervisor export and stable
host-edge surfaces stay on one Linux plugin vocabulary.
