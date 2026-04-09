# 005 - Real LV2 Discovery, Extension Negotiation, And Linux Proof

Status: complete
Owner: core-product
Created: 2026-04-08
Depends on: g09.002
Vision tags: `PLUGIN`, `LV2`, `LINUX`
Contract refs: `038`, `055`, `072`

## Problem

Signal already carries LV2 contracts and Linux-native breadth claims, but the
actual LV2 adapter remains scaffolded and does not yet prove real bundle,
manifest, URID, worker, patch, and lifecycle behavior.

## Goals

- [x] implement real LV2 bundle and manifest traversal
- [x] realize bounded URID, worker, patch, and extension negotiation through
      runtime-owned receipts
- [x] prove one honest Linux-native LV2 execution path

## Non-Goals

- [ ] no LV2 UI embedding
- [ ] no distro-wide compatibility certification in this milestone

## Execution Plan

### Batch 5.1 - Bundle And Manifest Discovery

- [x] implement real LV2 scan roots, bundle traversal, and manifest parsing
- [x] map URI, class, feature, and port metadata into shared runtime discovery
      receipts
- [x] record malformed bundles and missing features as typed discovery results

### Batch 5.2 - Extension And Lifecycle Baseline

- [x] implement bounded URID, patch, and worker negotiation where required for
      baseline lifecycle and state behavior
- [x] instantiate LV2 plugins through the hardened sandbox process
- [x] map feature-negotiation and activation failures into runtime-owned
      lifecycle and fault receipts

### Batch 5.3 - Linux-Native Proof

- [x] prove the LV2 path through runtime receipts and stable server-host export
- [x] add focused Linux-native smoke or acceptance lanes for discovery and
      execution
- [ ] wire one LV2 scenario into the interactive demo substrate

## Acceptance Criteria

- [x] LV2 discovery is real bundle/manifest traversal, not scaffolding
- [x] baseline extension negotiation is explicit and runtime-owned
- [x] the Linux-native LV2 path is proven through shared receipts

## Risks And Mitigations

- Risk: extension scope balloons beyond a bounded lifecycle baseline.
- Mitigation: promote only the minimal URID/worker/patch depth needed for the
  shared contract.

- Risk: Linux-only detail drifts into host-local policy.
- Mitigation: require stable runtime and server-host proof before closing the
  milestone.

## Evidence Requirements

- [x] log each LV2 tranche
- [x] run `cargo check -p signal-plugin-lv2`
- [x] run `cargo check -p signal-runtime`
- [x] run `effigy health`

## Batch 5.1 Tranche 1 Outcome

`signal-plugin-lv2` production discovery no longer infers identity from bundle
names and then rehydrates scaffold records. The adapter now parses bundle-local
`manifest.ttl` metadata for plugin type id, plugin URI, vendor, version, I/O
shape, required features, supported extensions, and coarse feature class, then
projects that directly into the existing runtime-facing LV2 discovery DTOs.
Server-side internal and public LV2 proof roots now emit the same manifest
contract, so the stable server host-edge baseline and LV2 extension proofs are
exercising the real manifest-backed path instead of a parallel test-only bundle
name shortcut.

This tranche deliberately stops before malformed-bundle typing and before
worker or lifecycle execution depth. The next meaningful seam is to turn bad or
underspecified manifests into explicit discovery results instead of silent
skips, then decide whether baseline extension negotiation needs any adapter-side
realization beyond the current metadata-backed capability summary before moving
deeper into brokered LV2 execution.

## Batch 5.1 Tranche 2 Outcome

LV2 discovery no longer drops bad bundles on the floor. The adapter now emits
typed diagnostics for malformed manifests and unsupported required features, and
the server host threads those diagnostics into runtime-owned scan receipts so
both the observation JSON and stable host-edge LV2 proofs export the same
failure truth. This keeps the discovery boundary honest without widening into
execution depth yet: bad LV2 bundles are explicitly visible, but the bounded
extension and lifecycle contract is still the next major realization seam.

## Batch 5.2 Tranche 1 Outcome

LV2 extension posture is no longer only inferred from discovery metadata after
the fact. The adapter now emits a bounded extension-preparation record during
session planning, the server host records that as runtime-owned sandbox
lifecycle truth during prepare, and the observation/report surfaces export it
through both the sandbox lifecycle snapshot and the LV2 extension snapshot when
an LV2 sandbox actually exists. This still stops short of broker-backed LV2
execution, but the worker, URID, and patch posture for the prepared lane is now
owned by adapter preparation rather than pure discovery projection.

## Batch 5.2 Tranche 2 Outcome

LV2 preparation failures are now explicit and runtime-owned instead of
collapsing into an untyped request error. The adapter can carry a bounded
metadata-backed preparation fault mode for one unavailable negotiation case, the
server host maps that into sandbox lifecycle, fault, and prepared-negotiation
records, and the stable public LV2-extension proof now exercises a
worker-unavailable lane that exports `Unavailable` negotiation truth through the
same report surface as the healthy path. This is the right place to stop adding
micro-fault variants: the next major seam is replacing the remaining demo
broker lane with a real LV2 broker-backed execution path.

## Batch 5.2 Tranche 3 Outcome

The bounded LV2 broker lane is no longer piggybacking on `Demo`. The server
host now passes real LV2 bundle identity into the hardened sandbox process, the
broker discovers, instantiates, prepares, and tears down the LV2 bundle through
`signal-plugin-lv2`, and the stable broker-backed server-host LV2 proof now
asserts exported LV2 negotiation truth instead of only a generic
`broker:lease_attached` marker. This meaningfully closes the sandbox-baseline
seam: LV2 preparation and teardown are now adapter-backed in both the direct
and brokered host paths, while bounded execution is the next major depth seam.

## Batch 5.3 Tranche 1 Outcome

The broker-backed LV2 lane now exports one bounded real execution record instead
of stopping at prepare truth. The LV2 adapter carries an explicit block
processing record, the hardened sandbox broker exposes an attached-session LV2
execution command, and the server host records that execution summary back into
runtime-owned transport detail so the stable public broker-backed LV2 proof can
assert exported execution truth rather than only negotiated prepare state. This
is still intentionally bounded to one execution record, not a stream, but it
closes the biggest remaining fake seam in the broker-backed LV2 lane.

## Batch 5.3 Tranche 2 Outcome

The bounded LV2 broker execution lane now carries a short multi-block stream
instead of collapsing everything into one attached execution receipt. The
hardened sandbox broker exports three ordered LV2 block records, the server
host folds that attached-stream summary back into runtime-owned transport
detail, and the stable public broker-backed LV2 proof now asserts exported
stream order and last-block completion truth. This is a better place to stop:
the next major seam is no longer basic broker execution realism, but either one
recovery-owned LV2 execution path or the first Linux-native acceptance smoke.

## Batch 5.3 Tranche 3 Outcome

The broker-backed LV2 stream truth now survives the first recovery-owned public
lanes instead of only the healthy attach path. The stable server-host LV2 crash
recovery, deferred-teardown fault, and cleanup-retry proofs all now assert the
same exported LV2 execution markers as the healthy broker lane, so recovery no
longer collapses back to generic `broker:lease_attached` truth. That closes the
most important proof gap in `g09.005`: the next step is no longer another host
proof variant, but Linux-native acceptance smoke and closeout assessment.

## Batch 5.3 Tranche 4 Outcome

`g09.005` now has a focused Linux-native acceptance boundary instead of relying
on ad hoc proof commands. `effigy acceptance:linux-lv2-execution-boundary`
exercises the public runtime LV2 discovery and lifecycle surface plus the
stable server-host broker-backed healthy and recovery LV2 execution lanes, and
`signal-supervisor-tools` now exports the same focused boundary as
`signal.runtime.linux-lv2-execution-boundary`. That gives the milestone one
honest Linux LV2 acceptance surface grounded in real discovery, bounded
negotiation, broker-backed execution stream truth, and one recovery-owned host
path.

This is the right promotion point. The remaining deferred scope is no longer a
missing production seam inside the LV2 realization lane; it is broader demo and
later breadth work such as interactive substrate wiring, deeper atom-schema
coverage, or LV2 UI/editor behavior. Those belong to the demo milestones and
later Linux breadth work, not this realization milestone.

## Next Task

COMPLETED: `g09.005` is closed.

Next, start `g09.006` with one meaningful structural-repair batch: audit the
largest remaining shared execution and recovery duplication between
`signal-host-local`, `signal-host-server`, and `signal-runtime`, then land one
broad consolidation pass on the highest-leverage seam before widening further
host-facing behavior.
