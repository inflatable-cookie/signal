# 001 - Production Host-Assembly Wiring

Status: active
Owner: core-product
Created: 2026-08-17
Depends on: g10 closeout (stretch audit complete)
Vision tags: `PLUGINS`, `RUNTIME`, `INTEGRATION`
Governing refs: `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md`, `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`, `docs/architecture/system-inventory.md`

## Problem

Signal already hosts CLAP, VST3, AU, and LV2 through adapter crates,
`signal-plugin-sandbox`, and `signal-plugin-bridge`. Proof lives in bridge,
sandbox, and render-plane tests.

`signal-host-local` still stops short of a production consumer path:

- discovery and sandbox broker exercises record runtime receipts
- bridge backends are not wired into the Pulse-facing host assembly end to end
- render-plane plugin stages are not driven from the host assembly in the
  canonical consumer path
- host crate docs still understate what is and is not wired today

Loophole and other consumers need one honest assembly path from scan → placement
→ bridge backend → render-plane execution without reconstructing test-only wiring.

## Goals

- [ ] freeze the host-assembly integration contract: authority chain, placement
  tiers supported in v1 (`InProcess`, `DedicatedSandbox`), and explicit non-goals
- [ ] add a host-owned bridge backend factory on `LocalRuntimeHost` that can
  load, activate, and hand out `RenderPluginProcessor` handles for CLAP, VST3,
  AU, and LV2
- [ ] wire render-plane plugin stages through the host assembly for at least one
  offline proof path and one public host-edge proof path
- [ ] refresh host crate docs, architecture inventory, and front doors so they
  describe integration seams accurately

## Non-Goals

- [ ] rebuilding adapter hosting from scratch
- [ ] SharedSandbox tier implementation (see `g11.002`)
- [ ] product browser, preset, or workflow UX
- [ ] Loophole mixer/layout policy or Chorus realization
- [ ] graph successor or device-depth backlog items

## Execution Plan

### Batch 1.1 - Integration Contract Freeze

Status: complete

Scope: docs-only. No production-code edits.

- [x] write the host-assembly integration map:
  - `signal-host-local` orchestration boundaries
  - `signal-plugin-bridge` backend selection by `PluginIsolationTier`
  - render-plane `RenderPluginProcessor` ownership and lifetime
  - runtime-owned placement and supervisor receipts that must stay authoritative
- [x] name the v1 supported tiers (`InProcess`, `DedicatedSandbox`) and the
  rejected tier (`SharedSandbox` → typed error until `g11.002`)
- [x] list the public host-edge tests that must go green before Batch 1.4 closes
- [x] amend Contract `072` or add a short architecture note only if the map
  exposes a real contract gap; do not invent parallel authority

Acceptance for this batch:

- one operator-readable integration map exists under `docs/architecture/` or
  `docs/contracts/`
- `g11/README.md`, `strategic-runway.md`, and `system-inventory.md` point at it
- Batch 1.2 scope is bounded enough to execute without fresh planning decisions

Batch 1.1 closed 2026-08-17. Integration map:
`docs/architecture/production-host-assembly-integration.md`.

### Batch 1.2 - Bridge Backend Factory On LocalRuntimeHost

Status: ready

- [ ] add host-owned backend construction for in-process CLAP/VST3/AU/LV2
- [ ] add host-owned `ShmPluginProcessor` construction bound to existing broker
  sessions where `DedicatedSandbox` is selected
- [ ] surface typed failures for unsupported tiers, layouts, and missing discovery
  records
- [ ] keep runtime-owned placement and lifecycle receipts authoritative

### Batch 1.3 - Render-Plane Consumer Wiring

Status: blocked on Batch 1.2

- [ ] drive at least one offline render-plane plugin stage from the host assembly
- [ ] prove parameter/event/state handoff boundaries on that path
- [ ] document which render-plane entry points are in v1 scope vs deferred

### Batch 1.4 - Public Host-Edge Proof And Front-Door Closeout

Status: blocked on Batch 1.3

- [ ] extend or add `signal-host-local` public host-edge tests that exercise
  real bridge backends, not broker metadata-only sessions
- [ ] run `effigy validate` and record the commands actually executed
- [ ] refresh `LocalRuntimeHost` crate docs and architecture front doors
- [ ] close the milestone and name the product-pull gate for `g11.002`

## Acceptance Criteria

- [ ] a consumer can follow one documented path from host assembly to real plugin
  audio through bridge backends
- [ ] v1 explicitly supports `InProcess` and `DedicatedSandbox` only
- [ ] `SharedSandbox` remains a typed rejection until `g11.002`
- [ ] no doc surface claims plugin hosting is missing or discovery-only
- [ ] public host-edge proof exists beyond broker attach/exercise metadata

## Risks and Mitigations

- Risk: host assembly becomes a second plugin authority beside runtime/contracts.
- Mitigation: keep placement, lifecycle, and supervisor meaning runtime-owned;
  host code stays orchestration and backend construction only.

- Risk: wiring only one format leaves a false "production ready" story.
- Mitigation: Batch 1.2 requires all four adapter families before Batch 1.3
  closes.

- Risk: test-only wiring persists without consumer entry points.
- Mitigation: Batch 1.4 requires public host-edge proof on the same path Batch
  1.3 uses.

## Evidence Requirements

- [ ] one log per completed batch under `docs/logs/2026-08/` or later month
- [ ] Batch 1.1 records the integration map path and bounded Batch 1.2 scope
- [ ] later batches record validation actually run (`effigy validate`, targeted
  crate tests)
- [ ] milestone closeout updates `docs/roadmaps/g11/README.md`

## Next Task

Execute
`docs/roadmaps/g11/batch-cards/001-g11-001-bridge-backend-factory.md`.
