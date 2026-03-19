# 032 Canonical Multichannel Layout And Channel-Role Contract

Status: complete
Owner: core-product
Updated: 2026-03-16
Related contracts: `docs/contracts/001-shared-dsp-and-host-boundary.md`, `docs/contracts/008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`, `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`, `docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned multichannel vocabulary so layout,
channel-role, bus-intent, and custom-layout fallback meaning stop being
inferred from raw channel counts before `g07` widens sidechain, spatial,
Linux, and complex plugin-I/O depth.

## Authority hierarchy

Multichannel meaning has one authority chain:

1. this contract defines canonical layout identity, channel-role meaning,
   bus-intent meaning, and custom-layout fallback rules
2. `signal-primitives` remains the low-level audio buffer authority through
   `ChannelLayout`, but it does not by itself define richer routing or
   speaker-role semantics beyond `Mono`, `Stereo`, and raw counts
3. `signal-graph` owns graph-node bus endpoints, topology metadata, and
   adaptation behavior that must later consume the shared layout vocabulary
4. `signal-runtime` must own the typed observation, topology, hardware, and
   plugin-facing receipts that expose multichannel truth to hosts and
   supervisor consumers
5. host crates, adapters, and downstream products may contribute concrete
   device or plugin capabilities, but they must not redefine canonical
   layout or role meaning

If a channel-mapping claim cannot be explained through this contract, the
runtime-owned receipts that follow from it, and Signal-owned graph or hardware
surfaces, it is not yet part of the shared multichannel boundary.

## Existing anchors

Batch 1.1 freezes this vocabulary on top of the current narrow implementation
anchors instead of pretending multichannel semantics already exist:

- `crates/signal-primitives/src/lib.rs`
  - `ChannelLayout::Mono`
  - `ChannelLayout::Stereo`
  - `ChannelLayout::Count(ChannelCount)`
- `crates/signal-graph/src/lib.rs`
  - `GraphNodeBusEndpoint`
  - graph-node topology metadata and bus ids
  - mono-to-stereo and stereo-to-mono adaptation paths
- `crates/signal-runtime/src/interfaces.rs`
  - runtime execution-topology and external-I/O receipt families that will need
    richer layout and role meaning in Batch 1.2

This contract does not claim broader multichannel behavior is implemented yet.
It freezes the meaning the later runtime and graph work must obey.

## Shared vocabulary

### Canonical layout

A `canonical layout` is a Signal-owned layout identity that carries both a
stable layout name and a stable ordered channel-role list. A canonical layout
is stronger than raw channel count.

Batch 1.1 freezes this first bounded canonical family:

- `Mono`
- `Stereo`
- `Lcr`
- `Quad`
- `Surround5_0`
- `Surround5_1`
- `Surround7_1`

These names define shared layout meaning. They do not imply product mixer UX,
speaker artwork, or environment certification.

### Channel role

A `channel role` is the semantic meaning of one channel position inside a
canonical layout. Batch 1.1 freezes this first shared role vocabulary:

- `Mono`
- `FrontLeft`
- `FrontRight`
- `FrontCenter`
- `LowFrequencyEffects`
- `SideLeft`
- `SideRight`
- `RearLeft`
- `RearRight`
- `Discrete(index)` for count-preserving custom layouts that do not yet map to
  a canonical role set

No host, adapter, or product may reinterpret these roles once a runtime-owned
receipt declares them.

### Bus intent

`bus intent` means why a channel group exists in the routing graph, not just
how many channels it carries. Batch 1.1 freezes this first bounded bus-intent
family:

- `MainProgram`
- `AuxSend`
- `AuxReturn`
- `Sidechain`
- `HardwareInput`
- `HardwareOutput`
- `AnalysisTap`

Later runtime and graph surfaces may add more intents, but they must stay
additive and Signal-owned.

### Custom-layout fallback

A `custom-layout fallback` is the bounded way Signal handles layouts that do
not map cleanly onto one of the canonical families yet.

The required fallback rules are:

1. preserve raw channel count truth
2. do not silently relabel a custom layout as a canonical surround layout
3. expose `Discrete(index)` roles when role meaning is not yet standardized
4. keep adaptation or routing policy conservative until a later contract adds
   stronger meaning

### Layout portability

`layout portability` means whether a layout and role declaration survives graph,
runtime, hardware, and plugin boundaries without host-local reinterpretation.

Batch 1.1 only freezes the vocabulary. Portability proof belongs to later
`g07` batches once runtime, plugin, and hardware receipts actually carry the
new layout meaning.

## Rules

### Rule 1: channel count is not enough

Raw channel count may remain a primitive, but it is not the semantic authority
for multichannel routing once a canonical layout is known.

### Rule 2: canonical layouts must stay ordered

A canonical layout must imply one stable channel-role order. Hosts and adapters
must not reshuffle roles privately and still claim the same layout identity.

### Rule 3: fallback must stay explicit

If Signal cannot identify a canonical layout, it must say so explicitly through
custom-layout fallback rather than quietly guessing from channel count.

### Rule 4: sidechain and auxiliary meaning is bus intent, not layout meaning

`Sidechain`, `AuxSend`, and `AuxReturn` are bus intents layered on top of a
layout, not substitute layout identities.

### Rule 5: the contract stays reusable

This contract does not freeze product speaker-label UX, hardware calibration,
or immersive certification. It freezes the shared runtime and routing
substrate only.

## Deferred scope

Batch 1.1 intentionally leaves these out:

- immersive or object-based role vocabularies beyond the bounded canonical set
- automatic multichannel adaptation policy beyond the current mono/stereo
  implementation
- plugin-side multi-bus or multi-output realization
- Linux backend or hardware-specific speaker map quirks
- final public proof that downstream consumers can inspect multichannel truth

## Batch 1.1 outcome

Batch 1.1 freezes the first reusable multichannel authority line for Signal:

- canonical layouts are now stronger than raw channel counts
- channel-role and bus-intent meaning are now explicit Signal-owned vocabulary
- custom-layout fallback is now conservative and explicit instead of implied
- later `g07` batches can widen runtime, sidechain, spatial, Linux, and
  complex plugin-I/O depth without reopening the base vocabulary question

## Batch 1.2 outcome

Batch 1.2 applies that frozen vocabulary to the first real runtime-owned
receipt family:

- runtime execution topology now carries raw `ChannelLayout` plus canonical
  layout, channel-role, and bus-intent meaning per planned and summarized node
- host hardware and external-I/O receipts now carry explicit input and output
  channel counts and canonical multichannel summaries instead of only output
  count
- plugin discovery and plugin-chain stage receipts now surface default
  multichannel input and output meaning without adapter-local reconstruction
- `Mono` is now an explicit shared channel role so the canonical `Mono` layout
  no longer has to fall back to discrete custom-role treatment

## Batch 1.3 outcome

Batch 1.3 proves the canonical multichannel substrate is consumable as a shared
boundary instead of only a widened internal receipt family:

- public runtime proof now verifies canonical layout, channel-role, bus-intent,
  and plugin default multichannel-I/O truth through runtime-owned reexports
- stable local and server host-edge proofs now verify the same multichannel
  receipts survive `supervisor_report()` without host-local reinterpretation
- `signal-supervisor-tools --describe-multichannel-boundary` now documents the
  shared multichannel seam in machine-readable form
- `effigy acceptance:multichannel-boundary` now provides the repo-owned proof
  task for the canonical multichannel boundary

This contract is now closed as the reusable base for later sidechain,
multi-bus, spatial, Linux, and complex plugin-I/O work.

## Next Task

Continue `g07.002` with Batch 2.2 by materializing runtime-owned sidechain
source, target, attachment-policy, and fallback receipts across live and
offline routing surfaces without reopening host-local routing ownership.
