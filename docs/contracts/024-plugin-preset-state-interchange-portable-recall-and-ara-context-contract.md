# 024 Plugin Preset-State Interchange, Portable Recall, And ARA Context Contract

Status: complete
Owner: core-product
Updated: 2026-03-15
Related contracts: `docs/contracts/007-plugin-backend-and-host-neutral-delegation-contract.md`, `docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md`, `docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md`, `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`, `docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`, `docs/architecture/package-map.md`

## Purpose

Freeze the first bounded preset-state, portable recall, and ARA-capable context
contract for `g06.013` so later runtime and adapter work can deepen recall,
state export, and clip-context transfer without reopening host-local ownership,
opaque preset blobs, or adapter-private ARA claims as if they were already
portable.

## Authority hierarchy

Preset/state interchange and ARA-capable context have one authority chain:

1. `signal-plugin` owns the shared plugin identity and capability substrate
   that later portable state/preset DTOs must extend from:
   - plugin format and type identity
   - generic parameter and event vocabulary
   - bounded processing contract and capability breadth
2. `signal-runtime` owns portable recall and interchange meaning for:
   - runtime recall payload/status and handoff snapshots
   - observation, supervisor, render, and delegated-execution surfaces
   - portable versus native-only fallback classes
   - bounded ARA-capable document, source, and region context descriptors once
     they exist
3. adapter crates such as `signal-plugin-clap`, `signal-plugin-vst3`, and
   `signal-plugin-au` own protocol-specific realization detail for:
   - native preset formats, blobs, and document references
   - adapter-specific state serialization or deserialization
   - ARA or host-protocol bridging detail not yet promoted into the shared
     Signal contract
4. host crates may broker storage, transport, and supervisor delivery, but
   they must not become the authority for:
   - portable versus native-only recall classification
   - preset-family portability claims
   - ARA-capable clip or document context meaning

If a state, preset, or ARA-context claim cannot be explained through
`signal-plugin`, `signal-runtime`, and additive Signal-owned receipts, it is
not yet part of the shared contract.

## Existing runtime anchors

This contract is grounded in current runtime-owned recall and export seams:

- `RuntimePluginRecallPayload`
- `RuntimePluginRecallSnapshot`
- `RuntimePluginRecallHandoffSnapshot`
- `RuntimePluginChainSnapshot`
- `RuntimeOfflinePluginExecutionBoundary`
- `RuntimeOfflineRenderContractPreview`
- observation and supervisor surfaces that already export recall state and
  payload snapshots

Batch 13.1 does not claim these anchors are already sufficient for portable
state interchange or ARA. It freezes how later DTOs and receipts must layer on
top of them.

## Shared vocabulary

This contract freezes the first bounded vocabulary for `g06.013`.

### Recall payload

`recall payload` keeps its existing Signal-owned meaning:

- runtime continuity and lifecycle evidence for a plugin stage
- plugin identity, sandbox identity, lifecycle, transport, and recovery data
- authoritative runtime recall status independent of export-only summaries

Recall payload is not yet the same thing as portable preset or state
interchange. It is the runtime-owned anchor that later portability receipts
must align to.

### Interchange payload

`interchange payload` means additive exported plugin state or preset meaning
that can cross runtime, host-edge, render, or downstream boundaries without
forcing consumers to parse adapter-native blobs just to know what class of
state they received.

An interchange payload may include:

- shared Signal-owned state fields
- native adapter supplement references
- preset-family identifiers or references
- explicit fallback or degradation class

### Preset descriptor

`preset descriptor` means a user-meaningful preset identity or family
reference, not authoritative plugin state by itself.

Preset descriptors may later carry:

- preset name or label
- preset family or source class
- user, factory, embedded, document, or transient origin
- whether the descriptor is portable, guarded, native-only, or unsupported

A preset descriptor must never be interpreted as proof of lossless state
interchange on its own.

### ARA-capable context

`ARA-capable context` is frozen as a bounded runtime-owned descriptor family
for later plugin-context transfer. It is intentionally narrower than a full
product clip editor or document model.

The first bounded ARA-capable context vocabulary is:

- `document context`
  - host/session-scoped document identity needed to explain one plugin-facing
    ARA environment
- `source context`
  - media-source identity and readiness needed to explain what clip or asset
    the plugin is being asked to inspect
- `region context`
  - bounded clip/region timing, extent, and musical-placement identity needed
    to explain which portion of source material is in scope

These descriptors are placeholders for later typed runtime DTOs. Batch 13.1
freezes their meaning so Batch 13.2 does not invent ARA semantics ad hoc.

## Portability and fallback classes

This milestone freezes five portability classes.

### Portable

`Portable` means Signal can explain the state or preset through shared
Signal-owned meaning without requiring adapter-private reconstruction by the
consumer.

Portable does not imply every adapter realizes the same fidelity. It means the
consumer can understand the recall outcome and what was transferred through
shared receipts.

### Guarded

`Guarded` means shared meaning exists, but realization depends on the
adapter/platform pair or on adapter-native supplement still being available.

Examples include:

- preset or state import that relies on one adapter-specific supplement while
  still exposing a shared portability outcome
- ARA-capable context whose bounded descriptor is shared, but whose live
  realization still depends on host and adapter support depth

### NativeOnly

`NativeOnly` means only the adapter-native blob, document, preset reference, or
protocol-specific state has authoritative meaning today.

Consumers may still receive a shared receipt that says the outcome is
`NativeOnly`, but they must not expect portable field-level interchange.

### ContextOnly

`ContextOnly` means Signal can describe the runtime-owned plugin context or
fallback state, but cannot yet claim portable plugin state interchange.

This class is especially important for early ARA-capable work, where document,
source, and region descriptors may become shared before preset or state
portability is solved.

### Unsupported

`Unsupported` means neither portable interchange nor bounded guarded fallback is
currently available through Signal-owned surfaces. Unsupported scope must stay
explicit in roadmap, descriptor, and receipt surfaces instead of being implied
by adapter breadth.

## Rules

### Rule 1: runtime recall remains the authority for portability outcomes

Products and hosts must not infer portable versus native-only recall from
adapter identity, preset file extension, or host storage location alone.

Portable recall classification must remain additive over runtime-owned recall
payload and later interchange receipts.

### Rule 2: preset references are descriptive, not authoritative state

Preset identifiers, factory slots, user labels, or document references may be
useful consumer surfaces, but they are not by themselves proof that:

- equivalent state can be reconstructed across adapters
- the same preset is losslessly portable across platforms
- a host can replace runtime-owned recall truth

### Rule 3: portable interchange must separate shared meaning from native supplement

If later runtime or export surfaces carry both shared state meaning and
adapter-native supplement, those layers must remain distinguishable.

Consumers must be able to tell:

- what portion of the recall outcome is genuinely shared
- what portion remains guarded or native-only
- whether a later restore or migration path depends on the native supplement

### Rule 4: ARA-capable context is bounded to document, source, and region identity

Batch 13.1 explicitly does not promote:

- full clip-editor workflow semantics
- region editing policy
- waveform editing ownership
- product-local arrangement or document persistence models

Later ARA-capable runtime work must stay on bounded document, source, and
region descriptors rather than drifting into product workflow ownership.

### Rule 5: hosts may broker storage and transfer, but not portability taxonomy

Hosts may later:

- store preset/state payloads
- pass interchange data across IPC or supervisor boundaries
- carry adapter-native blobs where required

They must not invent competing preset or ARA portability classes outside the
shared Signal-owned vocabulary.

## Adapter-private and deferred scope

Batch 13.1 intentionally keeps the following outside the shared contract:

- VST3 program lists, unit trees, preset document internals, and richer host
  context
- AU factory preset documents, user preset stores, parameter-tree depth, and
  Cocoa/editor context
- CLAP extension-specific state or recall semantics not yet promoted into the
  shared contract
- lossless interchange claims across CLAP, VST3, and AU
- full ARA protocol realization, waveform ownership, editing semantics, or
  persistent document models
- product-local preset browser, tagging, migration UX, or clip-editor workflow

Those areas may later gain additive Signal-owned surfaces, but they are not
promised by Batch 13.1.

## Batch 13.1 outcome

Batch 13.1 freezes the first bounded preset-state, portable recall, and
ARA-capable context boundary:

- Signal now has one shared vocabulary for portable, guarded, native-only,
  context-only, and unsupported recall outcomes instead of leaving preset/state
  portability implicit
- preset descriptors are explicitly separated from authoritative runtime recall
  and future interchange payloads
- ARA-capable work now has one bounded document/source/region context target
  instead of drifting into product-local editing semantics
- Batch 13.2 can now deepen runtime-owned recall and export surfaces on top of
  one fixed portability contract instead of reopening state ownership

## Batch 13.2 outcome

Batch 13.2 deepens the contract into runtime-owned receipt surfaces:

- `RuntimePluginRecallPayload` now carries typed interchange classification,
  optional preset descriptor, and optional bounded ARA document/source/region
  context instead of leaving portability implicit in lifecycle and recovery
  fields alone
- plugin lifecycle state now retains preset and ARA context so the same
  runtime-owned recall truth flows through plugin-chain snapshots, execution
  topology summaries, recall handoff snapshots, offline execution boundaries,
  and supervisor export
- stable host edges now forward the widened recall payload on
  `supervisor_report()` without inventing host-local preset or ARA portability
  taxonomy
- Batch 13.3 can now focus on consumer-facing proof and descriptor work rather
  than first inventing the portability DTO family

## Batch 13.3 outcome

Batch 13.3 closes the bounded portable recall consumer proof surface:

- downstream-style runtime tests now consume preset portability classes,
  preset descriptors, and bounded ARA document/source/region context through
  shared plugin-chain and recall-handoff receipts
- both stable host edges now prove that `supervisor_report()` exports the same
  runtime-owned portability and bounded ARA-context truth without adapter-local
  preset parsing or host-owned portability taxonomy
- `signal-supervisor-tools` now exposes the
  `signal.runtime.recall-portability-boundary` descriptor and repo-owned
  `effigy acceptance:recall-portability-boundary` task so consumers can inspect
  the proof surface without private host glue
- later device-supervision and hardware recovery work can now build on one
  closed preset-state interchange and portable recall baseline instead of
  reopening portability ownership

## Next Task

Continue `g06.014` with Batch 14.1 by freezing the runtime-owned device
supervision, restart-state machine, exhaustion, and fault-boundary contract
before deeper hardware recovery depth begins.
