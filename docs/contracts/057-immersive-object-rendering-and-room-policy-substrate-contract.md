# 057 Immersive Object Rendering And Room-Policy Substrate Contract

Status: complete
Owner: core-product
Updated: 2026-03-19
Related contracts: `docs/contracts/036-spatial-adapter-execution-contract.md`, `docs/contracts/037-surround-bed-object-and-mix-policy-expansion-contract.md`, `docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`, `docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md`, `docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md`, `docs/contracts/056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned immersive object rendering and
room-policy boundary so later immersive execution, monitoring, and export work
builds on one runtime-owned meaning instead of renderer-private room rules,
host-local deployment heuristics, or product-local immersive-console policy.

## Authority hierarchy

Immersive object rendering and room-policy meaning has one authority chain:

1. this contract defines immersive object-rendering posture, room-policy class,
   room-policy authority, and immersive room outcome
2. `docs/contracts/036-spatial-adapter-execution-contract.md` remains the
   authority for baseline spatial adapter class, execution mode, target
   environment, control family, activation policy, and fallback posture that
   immersive work must widen rather than replace
3. `docs/contracts/037-surround-bed-object-and-mix-policy-expansion-contract.md`
   remains the authority for surround-bed class, object role, mix policy,
   render scope, and expanded fallback meaning that immersive room policy must
   compose with rather than reopen
4. `docs/contracts/056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md`
   remains the authority for plugin-facing pin-group identity, dynamic
   bus-negotiation posture, and bounded routing fallback where immersive paths
   depend on richer plugin-routing topology
5. `docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md`
   remains the authority for LV2 worker, URID, patch, and extension
   negotiation detail where immersive-capable plugins expose richer renderer or
   room-policy controls
6. `docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`
   and `docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md`
   remain the authority for live backend ownership, session-role, device-claim,
   and stream-policy truth that immersive room policy must layer on top of
   instead of replacing
7. `signal-graph` owns node-local graph identity and any later structural
   immersive metadata needed to stage object-aware or room-aware execution
8. `signal-runtime` must own the typed execution, observation, diagnostic, and
   render receipts that expose immersive object and room-policy truth to hosts
   and supervisor consumers
9. hosts, renderers, adapters, and downstream products may contribute concrete
   speaker-device detail, room calibration data, or authoring UX, but they
   must not redefine immersive object or room-policy meaning once runtime-owned
   receipts exist

If an immersive rendering claim cannot be explained through this contract, the
closed spatial, surround/object, plugin-routing, LV2, Linux session, and
runtime-owned topology receipts, it is not yet part of the shared immersive
boundary.

## Existing anchors

Batch 6.1 freezes this contract on top of the current bounded implementation
anchors instead of pretending room-aware immersive runtime depth already
exists:

- `docs/contracts/036-spatial-adapter-execution-contract.md`
  - baseline spatial adapter execution, target environment, and fallback
    meaning that immersive work must widen rather than replace
- `docs/contracts/037-surround-bed-object-and-mix-policy-expansion-contract.md`
  - bed, object, mix-policy, render-scope, and expanded-fallback vocabulary
    that immersive room-policy meaning must obey
- `docs/contracts/056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md`
  - plugin-facing routing and dynamic bus truth that immersive object paths may
    depend on
- `docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md`
  - bounded LV2-native capability and extension meaning for immersive-capable
    plugins
- `docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`
  - live backend session and restart meaning that room-policy continuity must
    compose with
- `docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md`
  - Linux parity meaning for session-role, device-claim, and stream-policy
- `crates/signal-runtime/src/interfaces.rs`
  - current spatial execution summaries, richer-spatial receipts, complex-I/O
    snapshots, and observation/supervisor export seams that later batches must
    widen with immersive room-policy truth
- `crates/signal-runtime/src/runtime.rs`
  - current routing, richer-spatial fallback, and report projection seams that
    define the bounded baseline this contract expands from

This contract does not claim room-aware immersive execution is already
implemented. It freezes the meaning later runtime and renderer work must obey.

## Shared vocabulary

### Immersive object-rendering posture

An `immersive object-rendering posture` is the runtime-owned description of how
Signal expects object material to survive relative to the current room-policy
boundary.

Batch 6.1 freezes this bounded family:

- `NotRequested`
- `MetadataOnly`
- `RoomPolicyAware`
- `CollapsedToBed`
- `Unavailable`

This posture is stronger than raw object presence. It says whether objects are
expected to remain actionable against a room-policy boundary or are already
collapsed back into bed-oriented meaning.

### Room-policy class

A `room-policy class` is the runtime-owned category of the room behavior Signal
is trying to satisfy.

Batch 6.1 freezes this bounded family:

- `NoRoomPolicy`
- `ReferenceRoom`
- `MonitoringRoom`
- `DeploymentRoom`
- `FallbackRoom`

Room-policy class is not a product editor label and not a speaker-calibration
blob. It describes the reusable room-policy intent that later runtime and
renderer work must expose.

### Room-policy authority

`room-policy authority` means where the currently active room-policy meaning is
allowed to come from.

Batch 6.1 freezes this bounded family:

- `RuntimeDefault`
- `RuntimeDeclared`
- `HostForwarded`
- `RendererAdvisory`

This keeps the shared ownership line explicit. Hosts and renderers may forward
detail, but the shared room-policy boundary must still reduce to one
runtime-owned authority answer.

### Immersive room outcome

An `immersive room outcome` is the runtime-owned result when immersive object
and room-policy intent is projected onto the currently available renderer,
routing, and live-backend substrate.

Batch 6.1 freezes this bounded family:

- `RenderObjectsAgainstRoomPolicy`
- `PreserveObjectMetadataOnly`
- `CollapseObjectsIntoBed`
- `BypassRoomPolicy`
- `TerminalImmersiveFailure`

This outcome is distinct from richer-spatial mix policy. It explains what
happened at the room-policy boundary, not just whether objects existed in the
abstract.

## Rules

### Rule 1: immersive room policy must stay runtime-owned

Immersive object-rendering posture, room-policy class, room-policy authority,
and immersive room outcome belong to runtime-owned receipts, not to renderer
capability tables, host-local device heuristics, or product-local immersive UX.

### Rule 2: immersive meaning must compose with the closed spatial substrate

This contract widens the closed spatial and richer-spatial seams. It must not
replace canonical layout, bed/object role, mix-policy, or plugin-routing truth
with a second immersive taxonomy.

### Rule 3: renderer-private room detail stays advisory

Concrete speaker positions, calibration payloads, vendor object schemas, and
renderer-native room metadata may exist internally, but the shared boundary
must remain grounded in runtime-owned immersive posture, room-policy class,
authority, and outcome.

### Rule 4: live, offline, and diagnostic surfaces must converge

Later immersive work may stage rollout, but it must not create one room-policy
model for live execution, another for offline render, and a third for
observation or supervisor export.

### Rule 5: deployment and monitoring detail stay additive for now

This contract may acknowledge `MonitoringRoom` and `DeploymentRoom` as bounded
policy classes, but it does not freeze full speaker deployment, fold-down, or
monitor-scene semantics. Later milestones must widen those explicitly.

### Rule 6: product-local room design stays out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze immersive-console UX, room editors, speaker-layout design flows, or
consumer-facing monitoring workflows.

## Deferred scope

Batch 6.1 intentionally leaves these out:

- final runtime execution, render, and observation receipts for immersive room
  policy
- public runtime, supervisor, and host-edge proof surfaces
- full speaker deployment, fold-down, and monitoring-scene semantics
- renderer-capability negotiation or immersive export packaging
- room calibration, headphone virtualization, or renderer-specific payload
  schemas
- product-local immersive authoring or room-design UX

## Batch 6.1 outcome

Batch 6.1 freezes the first reusable immersive object and room-policy
authority line for Signal:

- immersive object-rendering posture, room-policy class, room-policy
  authority, and immersive room outcome are now explicit Signal-owned
  vocabulary
- later runtime execution can widen on top of the closed spatial,
  richer-spatial, plugin-routing, LV2, and Linux live-ownership seams instead
  of inventing a parallel room-policy or renderer-policy model
- unsupported room-aware object paths and renderer gaps are now required to
  explain themselves through bounded immersive room outcome meaning rather than
  host-local heuristics or renderer-private interpretation

## Batch 6.2 outcome

Batch 6.2 materializes the first runtime-owned immersive room-policy receipt
layer on top of that frozen contract:

- `signal-runtime` now carries one bounded `immersive_room_policy` summary on
  the existing richer-spatial execution surface used by execution topology,
  plugin-chain stages, and offline-render dependency preview instead of
  inventing a second immersive report family
- execution-topology and offline-render dependency preview now expose aggregate
  immersive counts for room-policy-bearing and fallback room-policy paths, so
  later monitoring or export work can build on one runtime-owned inspection
  seam
- the current realized baseline stays honest and explicit: canonical surround
  fallback paths now surface `FallbackRoom` plus `BypassRoomPolicy`, while
  stereo-only paths still remain outside the immersive room-policy seam

This batch still does not claim a true renderer-backed immersive path. It
freezes and materializes the first reusable room-policy substrate only.

## Batch 6.3 outcome

Batch 6.3 closes the focused public proof seam for that widened immersive
substrate:

- public runtime proofs now verify immersive room-policy receipts on execution
  topology, plugin-chain, and offline-render preview surfaces
- both stable host edges now forward the same runtime-owned immersive
  room-policy receipt family on supervisor export without host-local room
  reinterpretation
- `signal-supervisor-tools` now points the existing
  `signal.runtime.spatial-boundary` descriptor at this immersive room-policy
  contract and describes the widened topology, plugin-chain, and render-preview
  anchors instead of the earlier richer-spatial-only contract

This contract is now frozen and proven for the bounded immersive room-policy
seam. Later work may widen it, but it should not reopen ownership.

## Next Task

Open `g08.007` with Batch 7.1 by freezing the first runtime-owned speaker
deployment, fold-down, and monitoring-scene contract on top of the closed
immersive room-policy seam.
