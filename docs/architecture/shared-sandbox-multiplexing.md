# SharedSandbox Multiplexing

Status: active
Owner: core-product
Updated: 2026-08-17
Roadmap: `docs/roadmaps/g11/002-shared-sandbox-tier.md`
Contracts: `014`, `072`, `009`

## Purpose

Map SharedSandbox onto the **existing** sandbox broker protocol. Contract `014`
already owns grouping, blast radius, rebind, and receipts. This note freezes
the v1 implementation shape so `g11.002` does not invent a second process model.

No research lane. No new isolation vocabulary.

## Authority chain

```text
RuntimePluginPlacementPolicy  (Contract 014)
  -> isolation outcome SharedSandbox
  -> grouping key = plugin identity (v1)
       -> one broker child (stdio control transport)
            -> N hosted instances
                 -> one shm audio lease per instance
                      -> existing ShmPluginProcessor::attach
```

Runtime owns placement outcome and grouping interpretation. Host assembly
orchestrates the child and attaches leases. The broker realizes multiple
instances in one process. Bridge audio-thread code stays `ShmPluginProcessor`.

## Frozen v1 decisions

### Grouping key

Same **plugin type identity** shares one child.

- grouping key: `plugin:{plugin_type_id}`
- a placement rule may set an explicit `sandbox_group_key`; v1 host assembly
  still only multiplexes members that share that key **and** the same
  `plugin_type_id`
- vendor, format, and consumer-supplied keys stay out of v1
- DedicatedSandbox / IsolatedSandbox grouping stays `sandbox:{sandbox_id}`
  (one child, one instance)

Default placement remains IsolatedSandbox. SharedSandbox is selected only by
runtime placement policy or an explicit `PluginIsolationTier::SharedSandbox`
that the host records as that outcome — not by a host-local allowlist.

### Process and transport

| Piece | SharedSandbox v1 | DedicatedSandbox (unchanged) |
| --- | --- | --- |
| Child process | one per grouping key | one per instance |
| Control transport | existing stdio broker session | existing stdio broker session |
| Loaded plugins | `HashMap<instance_id, LoadedPlugin>` | single slot (`instance_id = sandbox_id`) |
| Audio shm | one region / lease per member | one region / lease |
| Bridge backend | reuse `ShmPluginProcessor` | `ShmPluginProcessor` |
| Child audio thread | one thread, poll member request stamps | one thread, one stamp |
| Blast radius | child crash / terminal covers all members | one instance |

Do not add a second `PluginBlockProcessor` type. Per-member audio is already a
shm lease.

### Wire compatibility

Keep every current command. Omitted `instance_id` means `sandbox_id`, which is
today's single-slot DedicatedSandbox path.

New commands (whitespace-separated, same v1 path rules):

```text
load-plugin-instance <instance_id> <library_path> <plugin_id>
activate-instance <instance_id> <sample_rate_hz> <min_frames> <max_frames>
unload-plugin-instance <instance_id>
deactivate-instance <instance_id>
```

Existing `load-plugin`, `activate`, `deactivate`, `unload-plugin`,
`set-parameters`, `open-editor`, and `close-editor` address the default
instance. Duplicate `instance_id` on load stays `plugin_already_loaded`.

`start-processing` / `stop-processing` stay **boundary-level**: one child
audio thread for all activated members. v1 does not add members after
`start-processing`. Sequence is load → activate (each member) → start once.

Receipts already carry `instance_id`. Member commands must echo the addressed
id. Child crash / `crashed` receipts remain boundary-level and apply to every
member.

### Client API

Extend `SandboxBrokerClientSession` with instance-addressed wrappers. Keep
the current methods as the default-instance path so DedicatedSandbox tests
do not move.

### Runtime receipts that must tell the shared story

Contract `014` Rule 5. When two members share a child, snapshots must show:

- `placement_outcome = SharedSandbox`
- `sandbox_group_key = plugin:{plugin_type_id}` (or the rule's key)
- `shared_boundary_member_count = N` (`N >= 2` in the multiplex proof)
- distinct per-instance lifecycle rows that still inherit one boundary
  continuity class
- child crash / terminal outcome explainable for **all** members without a
  host-private process map

`RuntimePluginPlacementPolicy` already exists. v1 SharedSandbox default group
key in `runtime_plugin_placement_decision` must become `plugin:{plugin_type_id}`
when the outcome is SharedSandbox and the rule omits `sandbox_group_key`.
Today it falls back to `sandbox:{sandbox_id}`, which would make sharing
impossible.

### Host assembly (`g11.001` factory)

`LocalRuntimeHost::prepare_plugin_processor(..., SharedSandbox)`:

1. require a discovered type
2. find or spawn the broker session for `plugin:{plugin_type_id}`
3. allocate a unique `instance_id`
4. `load-plugin-instance` / `activate-instance`
5. `start-processing` if the boundary is not yet running
6. `ShmPluginProcessor::attach` from that member lease
7. record runtime placement / member count; do not invent grouping in the host

Keep the frozen `prepare_plugin_processor` signature. `PluginIsolationTier`
maps onto `RuntimePluginIsolationOutcome`; the host does not grow a parallel
policy table.

## Tradeoffs vs DedicatedSandbox

- **Memory / CPU:** one child instead of N. Still N shm regions and N host
  `ShmPluginProcessor` maps. One child audio thread serializes members, so a
  slow plugin delays its siblings. That is the shared-boundary cost, not a
  bug.
- **Crash isolation:** a member fault can kill the child and every peer.
  DedicatedSandbox stays the default.
- **RT:** no extra shm hop vs DedicatedSandbox. The hop is still there; the
  saving is process count, not per-block transport.

## Proof surfaces

| Batch | Proof |
| --- | --- |
| 2.1 | two instances, same `plugin_type_id`, one child; two shm leases process; default-slot second `load-plugin` still `plugin_already_loaded`; DedicatedSandbox tests unchanged |
| 2.2 | host factory returns a real SharedSandbox handle; runtime snapshot shows grouping key and member count |
| 2.3 | child death / terminal is visible on both member receipts; docs stop saying unimplemented |

## Non-goals

- vendor or format grouping
- adding members after `start-processing`
- a new audio-thread backend
- replacing DedicatedSandbox as default
- product browser / trust UX
- Contract `014` vocabulary changes

## Next Task

Execute
`docs/roadmaps/g11/batch-cards/005-g11-002-broker-multiplexing.md`.
