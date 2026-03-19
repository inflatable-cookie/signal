# 035 Plugin Complex I/O Topology And Multi-Output Instrument Contract

Status: active
Owner: core-product
Updated: 2026-03-17
Related contracts: `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`, `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`, `docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md`, `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned complex plugin-I/O boundary so later
multi-output instrument, bus-capable FX, spatial, Linux, and adapter breadth
work build on one runtime-owned topology meaning instead of format-private bus
negotiation or host-local pin reconstruction.

## Authority hierarchy

Complex plugin-I/O meaning has one authority chain:

1. this contract defines plugin-port class, output topology, bus-capable FX
   class, attachment policy, and bounded fallback meaning
2. `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`
   remains the authority for canonical layout, channel-role, and bus-intent
   meaning that plugin I/O topology must layer on top of rather than replace
3. `docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md`
   remains the authority for secondary-input source, target, and fallback
   meaning where a plugin exposes sidechain-capable inputs
4. `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`
   remains the authority for bus-role, auxiliary-path, connection identity,
   and attachment class where a plugin participates in broader send, return,
   submix, or parallel topology
5. `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`
   remains the authority for cross-adapter parity bands and platform-scoped
   capability support; this contract must not redefine parity policy
6. `signal-plugin` and adapter crates may report concrete plugin capabilities,
   but they must not redefine complex I/O meaning once runtime-owned receipts
   exist
7. `signal-runtime` must own the typed discovery, execution, render, and
   supervisor receipts that expose complex plugin-I/O truth to hosts and
   downstream consumers

If a complex plugin-I/O claim cannot be explained through this contract, the
closed multichannel, sidechain, and multi-bus contracts, and runtime-owned
topology receipts, it is not yet part of the shared plugin-I/O boundary.

## Existing anchors

Batch 4.1 freezes this contract on top of the current bounded implementation
anchors instead of pretending multi-output runtime behavior is already deeper
than it is:

- `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`
  - bounded cross-adapter capability and platform-coverage meaning
- `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`
  - canonical layout, channel-role, custom-layout fallback, and bus-intent
    meaning
- `docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md`
  - secondary-input source, target, attachment policy, and fallback meaning
- `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`
  - bus-role, auxiliary-path, connection identity, attachment class, and
    fallback meaning
- `crates/signal-plugin/src/lib.rs`
  - plugin features, plugin I/O layout, and backend-neutral plugin capability
    vocabulary
- `crates/signal-plugin-vst3/src/lib.rs`
  - current VST3 instrument and utility layout coverage, including
    format-specific multi-output potential that later runtime batches must map
    onto shared meaning
- `crates/signal-plugin-au/src/lib.rs`
  - current AU instrument and utility layout coverage that later runtime
    batches must align to the same topology model
- `crates/signal-runtime/src/interfaces.rs`
  - current discovery, plugin-chain, execution-topology, and render-preview
    receipt families that later batches must widen with explicit complex
    plugin-I/O truth

This contract does not claim richer multi-output instruments or bus-capable FX
are fully realized yet. It freezes the meaning later runtime and adapter work
must obey.

## Shared vocabulary

### Plugin port class

A `plugin port class` is the runtime-owned purpose of one plugin-facing port
group within a wider topology.

Batch 4.1 freezes this bounded family:

- `MainInput`
- `MainOutput`
- `SecondaryInput`
- `AuxInput`
- `AuxOutput`
- `InstrumentOutput`
- `AnalysisOutput`

Port class is not raw channel count and is not format-private bus naming. It
describes why the plugin-facing port exists in Signal-owned routing terms.

### Complex plugin-I/O topology

A `complex plugin-I/O topology` is a runtime-owned declaration that a plugin
has more than one meaningful input or output group, or participates in one
explicit secondary-input, auxiliary, or multi-output relationship.

Batch 4.1 freezes these rules:

- a plugin topology must remain explicitly distinct from graph-global bus
  topology even when the two are aligned
- multiple outputs, secondary inputs, and auxiliary-capable ports must remain
  visible as declared topology rather than inferred from output count alone
- adapter-private bus names may exist internally, but shared runtime meaning
  must reduce them to bounded Signal-owned port classes and role receipts

### Multi-output instrument

A `multi-output instrument` is a plugin with instrument capability whose audio
output surface contains one primary program output plus one or more additional
declared output groups.

Batch 4.1 freezes these rules:

- the primary output must stay identifiable as the default instrument render
  path
- additional outputs must remain explicit and individually attachable rather
  than being flattened into one wide anonymous output
- multiple outputs do not imply arbitrary mixer UX or format-private pin
  matrices

### Bus-capable FX

A `bus-capable FX` is a non-instrument processing plugin that can participate
in richer input or output topology than a single main stereo path.

Batch 4.1 freezes this bounded class family:

- `SinglePathFx`
- `SidechainCapableFx`
- `SendReturnCapableFx`
- `ParallelCapableFx`
- `MultiStemFx`

Later batches may widen this family, but they must remain additive and
Signal-owned.

### Attachment policy

`attachment policy` means how strongly Signal expects a declared plugin-facing
port group to remain attached at execution time.

Batch 4.1 freezes this bounded family:

- `Required`
- `Optional`
- `Disabled`

This policy is applied to plugin-facing topology, not just graph-global bus
relationships.

### Fallback outcome

A `fallback outcome` is the runtime-owned result when a declared complex
plugin-I/O topology cannot be attached or cannot remain active.

Batch 4.1 freezes this bounded family:

- `CollapseToPrimaryPath`
- `BypassUnavailablePortGroup`
- `MuteDependentOutput`
- `SafeModeDegradation`
- `TerminalPluginTopologyFailure`

Later batches may add more precise outcomes, but they must remain additive and
runtime-owned.

## Rules

### Rule 1: plugin topology must stay explicit

Signal must not infer complex plugin-I/O only from raw audio input or output
counts once shared topology receipts exist.

### Rule 2: instrument outputs stay distinct from generic bus fan-out

Multi-output instrument meaning must preserve the difference between the
default instrument program path and additional declared output groups.

### Rule 3: bus-capable FX must reuse the routing substrate

Sidechain-capable, send-return-capable, and parallel-capable FX must layer on
top of the closed sidechain and multi-bus contracts rather than inventing a
plugin-only routing taxonomy.

### Rule 4: adapter-private pin names remain advisory

Adapters may retain format-native bus, pin, or stem names internally, but the
shared boundary must remain grounded in runtime-owned port class, attachment
policy, and fallback meaning.

### Rule 5: product routing UX stays out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze mixer-strip presentation, pin-matrix UX, editor workflow, or product
console policy.

## Deferred scope

Batch 4.1 intentionally leaves these out:

- final runtime execution and render receipts for complex plugin-I/O
- final adapter negotiation proofs for CLAP, VST3, and AU on richer multi-bus
  plugins
- spatial or immersive plugin-output semantics
- product-local pin editing, mixer assignment, or bus-color policy
- arbitrary distributed or remote plugin routing behavior

## Batch 4.1 outcome

Batch 4.1 freezes the first reusable complex plugin-I/O authority line for
Signal:

- plugin port class, complex plugin-I/O topology, multi-output instrument, and
  bus-capable FX meaning are now explicit Signal-owned vocabulary
- richer instrument and FX routing can now build on the closed multichannel,
  sidechain, and multi-bus substrate instead of reopening routing semantics
  per format
- adapter capability receipts now have one bounded shared target for later
  runtime and proof work rather than drifting into CLAP, VST3, or AU private
  bus terminology

## Batch 4.2 outcome

Batch 4.2 now materializes this contract on runtime-owned receipts instead of
leaving complex plugin-I/O frozen only in prose.

The shared runtime boundary now carries:

- `RuntimePluginComplexIoSummary` on discovered plugin-type records
- complex plugin-I/O counts in format coverage and capability coverage receipts
- complex topology on plugin-chain stage snapshots used by execution and recall
  surfaces
- offline render dependency previews that now enumerate complex plugin-I/O
  stages, multi-output instruments, and bus-capable FX counts

The bounded adapter baseline widened at the same time: VST3 and AU fixtures now
expose one multi-output instrument and one bus-capable FX path so the runtime
surface is proven against richer plugin topologies instead of only simple
instrument or utility layouts.

## Batch 4.3 outcome

Batch 4.3 closes the bounded consumer-proof seam for this contract.

The shared runtime boundary is now proven through:

- public runtime proof for discovery, plugin-chain, and offline render
  dependency preview receipts
- stable local and server host-edge proof for forwarded complex plugin-I/O
  topology on supervisor export
- a machine-readable `signal.runtime.complex-io-boundary` descriptor and
  repo-owned acceptance task

This means complex plugin-I/O, multi-output instrument, and bus-capable FX
meaning now remains consumable without adapter-local pin reconstruction across
the shared runtime and host-edge surfaces that downstream consumers actually
use.

## Next Task

Continue `g07.006` with Batch 6.2 by materializing runtime-owned surround-bed,
object-role, mix-policy, render-scope, and expanded-fallback receipts across
execution, render, and observation surfaces without reopening host-local or
renderer-local spatial ownership.
