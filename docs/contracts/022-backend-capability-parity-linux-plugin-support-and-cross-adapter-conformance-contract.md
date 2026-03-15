# 022 Backend Capability Parity, Linux Plugin Support, And Cross-Adapter Conformance Contract

Status: complete
Owner: core-product
Updated: 2026-03-15
Related contracts: `docs/contracts/008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`, `docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md`, `docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`, `docs/architecture/package-map.md`

## Purpose

Freeze the first cross-adapter parity contract for `g06.011` so later runtime
parity and conformance work can widen CLAP, VST3, and AU capability depth
without reopening host-local portability claims, Linux support ambiguity, or
adapter-private behavior as if it were already portable.

## Authority hierarchy

Cross-adapter parity has one authority chain:

1. `signal-plugin` owns the format-neutral plugin vocabulary for:
   - plugin identity and feature class
   - default audio/event I/O shape
   - state, processing, lifecycle, readiness, and fault meaning
2. `signal-runtime` owns the active parity interpretation through:
   - discovery snapshots and scan receipts
   - format coverage and capability coverage summaries
   - lifecycle, chain, recall, interruption, fault, and delegated-execution
     receipts
   - report and supervisor export delivery
3. adapter crates such as `signal-plugin-clap`, `signal-plugin-vst3`, and
   `signal-plugin-au` own protocol-specific realization detail:
   - traversal and discovery helpers
   - backend-native initialization and transport hooks
   - extension, packet, event, or host-context realization
4. host crates may request scans, broker sandbox transport, or expose runtime
   reports, but they must not become the authority for:
   - portable versus adapter-private capability claims
   - Linux support claims
   - unsupported-platform conclusions when shared runtime-owned receipts or
     this contract already define the answer

If a portability or parity conclusion cannot be explained through
`signal-plugin`, `signal-runtime`, and additive Signal-owned receipts, it is
not yet part of the shared contract.

## Parity bands

This contract freezes four consumer-visible parity bands.

### Portable now

A capability belongs to the portable band when consumers may rely on the same
Signal-owned meaning across CLAP, VST3, and AU without format-specific
reclassification.

The portable band currently includes:

- shared backend identity through `PluginFormat`
- format-neutral descriptor, feature, default audio/event I/O, state,
  processing, readiness, and lifecycle meaning
- runtime-owned discovery/catalog delivery through:
  - `RuntimePluginDiscoverySnapshot`
  - `RuntimePluginScanReceipt`
  - `RuntimePluginDiscoveredTypeRecord`
  - `RuntimePluginFormatCoverageRecord`
  - `RuntimePluginCapabilityCoverageSummary`
- runtime-owned lifecycle, continuity, and sandbox/export delivery through:
  - `RuntimePluginLifecycleSnapshot`
  - `RuntimeObservationReport`
  - `RuntimeSupervisorReport`
- format-scoped placement identity through shared plugin/sandbox surfaces rather
  than host-local adapter ledgers

Portable means shared Signal-owned interpretation, not identical adapter
internals.

### Portable with format guard

A capability belongs to the guarded band when it uses the same shared Signal
receipt family, but consumers must inspect backend or platform identity before
assuming it is available.

The guarded band currently includes:

- Linux-hosted plugin breadth:
  - CLAP may participate
  - VST3 now participates through explicit Linux-hosted scan/load coverage
  - AU is explicitly unsupported because it remains macOS-scoped
- shared-sandbox and placement-policy decisions where runtime-owned receipts
  may depend on `PluginFormat`
- future additive unsupported-platform and fallback receipts that will stay in
  the same runtime-owned discovery/lifecycle/export family rather than a
  separate host taxonomy

Guarded means consumers may rely on one shared vocabulary, but availability is
not universal across all formats or platforms.

### Adapter-private

A capability remains adapter-private when Signal has not yet promoted it into
shared DTOs or runtime-owned receipts.

Adapter-private today includes:

- CLAP-specific extension negotiation and packet detail
- VST3 unit, program-list, richer note-expression, and controller-side editing
  detail
- AU parameter-tree, preset-document, editor, Cocoa-view, and richer host UI
  depth
- backend-native traversal, factory, property, and instantiation plumbing that
  does not yet change consumer-visible shared meaning

Consumers must not infer portability from adapter-private detail.

### Unsupported or deferred

A capability belongs to the unsupported or deferred band when this contract
explicitly says Signal has not yet promoted it into the portable or guarded
band.

Unsupported or deferred today includes:

- AU support on Linux or Windows
- any claim of full CLAP, VST3, and AU parity for richer event-model,
  parameter-tree, editor, preset, or unit-program depth
- any product-local fallback, wrapper, or browser behavior outside shared
  runtime-owned receipts

Unsupported or deferred scope must stay explicit in roadmap, contract, and
descriptor surfaces rather than being implied by adapter existence alone.

## Capability matrix

This contract freezes the first bounded parity matrix.

### Matrix column meanings

- `portable`: one shared Signal-owned meaning across CLAP, VST3, and AU
- `guarded`: shared Signal-owned meaning exists, but consumers must gate on
  format or platform identity
- `private`: adapter-specific detail, not yet promoted
- `unsupported`: explicitly not available for that format or platform

### Current matrix

| Capability family | CLAP | VST3 | AU | Notes |
| --- | --- | --- | --- | --- |
| Shared plugin identity, feature class, default I/O shape | portable | portable | portable | Owned by `signal-plugin` and runtime-owned discovery receipts |
| Discovery/catalog export through runtime and supervisor surfaces | portable | portable | portable | Backed by shared runtime-owned discovery DTOs |
| Lifecycle, continuity, and sandbox/export delivery | portable | portable | portable | Uses one runtime-owned lifecycle and continuity family |
| Placement/isolation policy identity | guarded | guarded | guarded | Meaning is shared, but policy may depend on `PluginFormat` |
| Linux-hosted plugin support | guarded | guarded | unsupported | CLAP and VST3 may claim Linux breadth; AU remains macOS-scoped |
| Delegated execution / offline boundary identity | portable | portable | portable | Ownership stays runtime-owned even when adapters fulfill work |
| Rich event-model, editor, preset, unit-tree, or parameter-tree depth | private | private | private | Remains later cross-adapter work |

The matrix is intentionally bounded. It freezes what consumers may rely on
today without overstating backend parity that Signal has not yet proven.

## Linux support rule

This contract freezes one explicit Linux-support rule:

- Linux plugin support must be stated through runtime-owned breadth,
  discovery, lifecycle, and future unsupported-platform receipts
- support must never be inferred from adapter naming alone
- `PluginFormat::Au` does not imply Linux support
- later Linux parity claims for CLAP or VST3 must remain visible through the
  same shared runtime-owned receipt family rather than host-local platform
  conditionals

## Fallback and unsupported-state rule

This milestone does not yet require every unsupported or degraded parity case
to have a dedicated DTO. It does require one bounded rule:

- unsupported, unavailable, or guarded parity conclusions must land in
  runtime-owned discovery, lifecycle, export, or additive coverage receipts
  once promoted
- host crates and downstream tools must not invent a second unsupported-state
  taxonomy to explain backend or platform breadth

Batch 11.2 must deepen this rule into richer runtime-owned receipts where the
current shared coverage summaries are not yet explicit enough.

## Conformance rule

Cross-adapter conformance must be proven in this order:

1. public runtime receipt consumption
2. stable host-edge `supervisor_report()` delivery
3. machine-readable supervisor-tools descriptor or acceptance surface

Consumers should not need adapter crate internals, host-private scan ledgers,
or product-local portability matrices to answer the bounded parity questions
frozen here.

## Explicit deferred scope

Batch 11.1 intentionally does not claim:

- full event-model parity across CLAP, VST3, and AU
- full preset, editor, unit-tree, or parameter-tree parity
- runtime-owned unsupported-platform receipts for every backend edge case
- release or marketing feature matrices detached from runtime-owned evidence

Those belong to later `g06.011`, `g06.012`, or `g06.013` work.

## Batch 11.1 outcome

Batch 11.1 freezes the first bounded cross-adapter parity boundary:

- consumers now have one Signal-owned parity vocabulary across CLAP, VST3, and
  AU
- Linux plugin breadth is now explicitly classified as guarded rather than
  implied by adapter existence
- portable, guarded, adapter-private, and unsupported scope are now separated
  clearly enough for later runtime receipt work and conformance proof
- Batch 11.2 can now deepen runtime-owned parity receipts without reopening the
  meaning of portability itself

## Batch 11.2 outcome

Batch 11.2 deepens the parity contract into runtime-owned receipts:

- `RuntimePluginScanReceipt`, `RuntimePluginDiscoverySnapshot`, and
  `RuntimePluginLifecycleSnapshot` now share typed per-format parity coverage
  instead of forcing consumers to reconstruct platform scope and placement
  alignment from separate fields
- hosts now record explicit platform coverage for CLAP, VST3, and AU so Linux
  breadth and AU macOS-only scope are visible through runtime-owned receipts
- failure, continuity, active-transport, and format-scoped placement-rule
  counts now align on one parity record family rather than adapter-local
  reasoning
- Batch 11.3 can now prove consumer-facing conformance against one bounded
  shared receipt vocabulary

## Batch 11.3 outcome

Batch 11.3 closes the first cross-adapter consumer proof surface:

- downstream-style runtime tests now consume parity bands plus supported and
  unsupported platform scope through shared discovery and lifecycle receipts
- both stable host edges now prove that `supervisor_report()` forwards the same
  parity receipt family without host-private portability tables
- `signal-supervisor-tools` now exposes the
  `signal.runtime.cross-adapter-parity-boundary` descriptor and repo-owned
  acceptance path so consumers can inspect the proof surface without adapter
  crate internals
- later `g06.012+` work can build richer event or preset depth on top of this
  bounded parity baseline instead of reopening backend breadth authority

## Next Task

Continue `g06.012` with Batch 12.1 by freezing the widened generic MIDI,
note-expression, and plugin-event vocabulary across CLAP, VST3, and AU before
runtime and adapter event-depth work begins.
