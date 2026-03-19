# 039 Linux Cross-Adapter Plugin Parity And Sandbox Policy Contract

Status: complete
Owner: core-product
Updated: 2026-03-17
Related contracts: `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`, `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`, `docs/contracts/038-lv2-adapter-baseline-and-linux-native-plugin-lifecycle-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first Linux-specific cross-adapter parity and sandbox-policy
boundary for `g07.008` so later runtime receipt work can widen CLAP, VST3, and
LV2 Linux breadth without reopening host-local portability claims, Linux-only
wrapper behavior, or adapter-private sandbox policy as if it were already
portable.

## Authority hierarchy

Linux plugin parity has one authority chain:

1. `signal-plugin` owns the backend-neutral plugin vocabulary for:
   - backend identity
   - feature class
   - default audio and event I/O shape
   - bounded capability and lifecycle meaning
2. `signal-runtime` owns the Linux parity interpretation through:
   - discovery snapshots and scan receipts
   - lifecycle, continuity, render, failure, and placement receipts
   - sandbox grouping, rebind, interruption, and fallback classification
   - observation, supervisor, and acceptance export delivery
3. adapter crates such as `signal-plugin-clap`, `signal-plugin-vst3`, and
   `signal-plugin-lv2` own protocol-specific realization detail:
   - native scan roots and traversal
   - format-native load or instantiate helpers
   - extension, feature, manifest, or URI projection
   - backend-native transport and session-planning detail
4. host crates may request scans, broker sandbox transport, or expose shared
   reports, but they must not become the authority for:
   - Linux portability claims
   - shared-sandbox policy meaning
   - unsupported Linux adapter conclusions
   - plugin fallback classification when runtime-owned receipts already define
     the answer

If a Linux parity or sandbox-policy conclusion cannot be explained through
Signal-owned plugin and runtime receipts, it is not yet part of the shared
contract.

## Linux parity bands

This contract freezes four Linux-facing parity bands.

### Portable on Linux

A capability belongs to the portable Linux band when consumers may rely on the
same Signal-owned meaning across CLAP, VST3, and LV2 on Linux without
format-specific reinterpretation.

The portable Linux band currently includes:

- shared backend identity through `PluginFormat`
- runtime-owned discovery and scan delivery through:
  - `RuntimePluginDiscoverySnapshot`
  - `RuntimePluginScanReceipt`
  - `RuntimePluginDiscoveredTypeRecord`
  - `RuntimePluginFormatCoverageRecord`
  - `RuntimePluginCapabilityCoverageSummary`
- runtime-owned lifecycle and continuity meaning through:
  - `RuntimePluginLifecycleSnapshot`
  - `RuntimeInterruptionSummary`
  - `RuntimeDegradationSummary`
- runtime-owned sandbox and placement-policy meaning through:
  - `RuntimePluginLifecycleSnapshot`
  - plugin continuity and shared-sandbox receipts already frozen in `g06.003`
- observation, supervisor export, and repo-owned acceptance delivery

Portable on Linux means one shared Signal-owned interpretation, not identical
adapter internals or identical extension depth.

### Portable with Linux policy guard

A capability belongs to the guarded Linux band when it uses the same Signal
receipt family, but consumers must inspect adapter, policy, or fallback state
before assuming identical behavior.

The guarded Linux band currently includes:

- placement-policy outcomes where runtime-owned receipts may choose:
  - `InProcess`
  - `SharedSandbox`
  - `IsolatedSandbox`
- shared-sandbox grouping and rebindability, because the policy answer may
  differ by adapter capability, failure history, or sandbox rule match
- Linux plugin fallback conclusions such as:
  - supported but downgraded placement
  - supported but restartable interruption exposure
  - supported but typed unsupported-feature or unavailable-extension outcomes
- future additive unsupported-platform or unsupported-extension receipts that
  stay inside the same runtime-owned discovery, lifecycle, and export family

Guarded means consumers may rely on one vocabulary, but they must inspect the
runtime-owned policy or fallback answer instead of assuming universal behavior.

### Adapter-private on Linux

A capability remains adapter-private on Linux when Signal has not yet promoted
it into shared DTOs or runtime-owned receipts.

Adapter-private on Linux currently includes:

- CLAP-specific extension negotiation and packet detail
- VST3 unit, program-list, controller-editing, and richer note-expression
  detail
- LV2 worker, UI, patch, URID, atom, and richer extension semantics
- format-native bundle, manifest, factory, property, or scan-root traversal
  that does not yet change shared consumer meaning
- adapter-internal process model or IPC detail that does not alter the shared
  runtime sandbox outcome

Consumers must not infer Linux portability or sandbox semantics from
adapter-private detail.

### Unsupported or deferred on Linux

A capability belongs to the unsupported or deferred Linux band when this
contract explicitly says Signal has not yet promoted it into the portable or
guarded band.

Unsupported or deferred on Linux currently includes:

- AU as part of the Linux plugin story
- any claim of identical extension depth across CLAP, VST3, and LV2
- any Linux feature matrix detached from runtime-owned receipts
- host-local scan policy, wrapper policy, or unsupported-state taxonomies
- broader ALSA, JACK, or PipeWire backend parity, which belongs to later `g07`
  hardware milestones rather than this plugin-parity batch

Unsupported or deferred scope must stay explicit rather than being implied by
adapter existence alone.

## Linux parity matrix

This contract freezes the first bounded Linux plugin matrix.

### Matrix column meanings

- `portable`: one shared Linux-facing Signal-owned meaning across CLAP, VST3,
  and LV2
- `guarded`: shared Signal-owned meaning exists, but consumers must inspect
  runtime-owned policy or fallback state
- `private`: adapter-specific detail, not yet promoted
- `unsupported`: explicitly not part of the current Linux plugin surface

### Current matrix

| Capability family | CLAP | VST3 | LV2 | Notes |
| --- | --- | --- | --- | --- |
| Linux discovery and catalog export through runtime receipts | portable | portable | portable | Shared discovery and format-coverage receipts now cover all three adapters |
| Linux lifecycle, continuity, and export delivery | portable | portable | portable | Uses one runtime-owned lifecycle and continuity family |
| Placement and sandbox-policy identity | guarded | guarded | guarded | Meaning is shared, but the runtime policy answer may differ |
| Shared-sandbox grouping and rebindability | guarded | guarded | guarded | Grouping is runtime-owned; identical adapter internals are not assumed |
| Linux failure, interruption, and fallback classification | guarded | guarded | guarded | Consumers inspect runtime-owned receipts rather than infer from format |
| Rich adapter extension depth | private | private | private | Still later cross-adapter work |
| AU-style non-Linux backend claims | unsupported | unsupported | unsupported | Outside this Linux adapter set |

The matrix is intentionally bounded. It freezes what consumers may rely on now
without pretending Linux adapter breadth is already feature-identical.

## Sandbox-policy rules

This contract freezes four Linux sandbox-policy rules.

### Rule 1: Linux plugin breadth reuses one shared sandbox contract

CLAP, VST3, and LV2 on Linux must reuse the existing Signal-owned placement,
shared-sandbox, isolation, continuity, interruption, and rebind vocabulary
already frozen in `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`.

Linux breadth may widen the adapter set, but it must not create a second
Linux-only sandbox taxonomy.

### Rule 2: runtime-owned policy beats host-local wrapper policy

When Linux adapters differ in crash isolation, restart behavior, or placement
eligibility, the authoritative answer must land in runtime-owned placement,
lifecycle, continuity, and failure receipts.

Host crates and downstream tools must not rebuild one Linux wrapper-policy
matrix from adapter internals, process names, or host-private ledgers.

### Rule 3: unsupported Linux behavior must stay typed and shared

If an adapter cannot satisfy a Linux policy expectation, the shared answer must
land as one runtime-owned guarded or unsupported outcome through discovery,
lifecycle, placement, continuity, or additive fallback receipts.

This milestone does not require every unsupported edge case to have a brand new
DTO. It does require unsupported Linux behavior to stay in the existing shared
receipt family instead of a second host-local unsupported taxonomy.

### Rule 4: parity does not imply extension identity

Linux parity means one shared consumer vocabulary for discovery, lifecycle,
sandbox policy, and bounded fallback. It does not mean CLAP, VST3, and LV2
share the same worker, UI, event, preset, or extension depth.

Later milestones may promote more of that depth additively, but Batch 8.1 does
not let products or hosts blur adapter-private extension gaps into shared
parity claims.

## Explicit deferred scope

Batch 8.1 intentionally does not claim:

- equal rich extension depth across CLAP, VST3, and LV2
- Linux hardware backend parity across ALSA, JACK, and PipeWire
- host-local wrapper or bridge policy outside shared runtime-owned receipts
- release or marketing matrices detached from runtime-owned Linux evidence

Those belong to later `g07.008+`, `g07.009`, and `g07.010` work.

## Batch 8.1 outcome

Batch 8.1 freezes the first bounded Linux cross-adapter parity and sandbox
boundary:

- CLAP, VST3, and LV2 now have one explicit Linux-facing parity vocabulary
  instead of separate adapter narratives
- sandbox and placement policy are now explicitly reused from the existing
  shared runtime-owned continuity contract rather than treated as Linux-host
  wrapper behavior
- portable, guarded, adapter-private, and unsupported Linux scope are now
  separated clearly enough for later runtime receipt work and proof
- Batch 8.2 can now deepen runtime-owned Linux parity receipts without
  reopening the meaning of Linux portability itself

## Batch 8.2 outcome

Batch 8.2 deepens the Linux parity contract into runtime-owned receipts:

- `RuntimePluginFormatPlatformCoverageRecord` and
  `RuntimePluginFormatParityRecord` now carry Linux-specific parity and policy
  meaning instead of forcing consumers to infer Linux portability from the
  broader cross-platform parity band alone
- runtime-owned discovery and lifecycle surfaces now align Linux render,
  placement, restart, rebindability, and failure posture through one shared
  record family rather than separate host-local summaries
- server-host Linux scan and sandbox paths now feed the widened parity record
  with explicit CLAP, VST3, and LV2 Linux policy defaults on the same runtime
  substrate
- Batch 8.3 can now close the downstream proof seam against one bounded Linux
  plugin vocabulary

## Batch 8.3 outcome

Batch 8.3 closes the downstream Linux parity proof seam:

- `signal-runtime` now proves the widened Linux parity and sandbox-policy
  receipt family is consumable through public runtime observation and
  supervisor surfaces without private host helpers
- the stable server host edge now proves it forwards the same Linux parity,
  placement, restart, rebindability, and failure vocabulary on
  `RuntimeSupervisorReport`
- `signal-supervisor-tools` now exposes
  `signal.runtime.linux-plugin-parity-boundary`, and Effigy now owns
  `acceptance:linux-plugin-parity-boundary`

This closes the bounded Linux plugin parity and sandbox-policy contract while
leaving Linux hardware backend portability, richer extension depth, and later
backend clocking work to subsequent `g07` milestones

## Next Task

Continue `g07.009` with Batch 9.1 by freezing the runtime-owned Linux audio
backend portability contract across ALSA, JACK, and PipeWire on top of the
now-closed Linux plugin parity and sandbox-policy boundary.
