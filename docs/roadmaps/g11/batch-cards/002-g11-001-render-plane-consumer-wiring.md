# 002 - g11.001 Render-Plane Consumer Wiring

Status: blocked
Owner: core-product
Updated: 2026-08-17
Master spec refs: none (baseline-routed; no active strict spec)
Roadmap refs: g11.001
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md, docs/architecture/production-host-assembly-integration.md, docs/roadmaps/g11/001-production-host-assembly-wiring.md, docs/roadmaps/g11/batch-cards/001-g11-001-bridge-backend-factory.md
Auto-start next card: yes
Depends on: 001-g11-001-bridge-backend-factory.md

## Objective

Drive at least one offline render-plane plugin stage from `LocalRuntimeHost`
using a `RenderPluginProcessor` created by `prepare_plugin_processor`, not by
test-only construction.

## Scope

- one offline proof path: host prepares a processor, a render-plane plan uses
  that handle as a plugin stage, audio is processed
- document which render-plane entry points are in v1 vs deferred
- parameter/event/state handoff only as far as the existing bridge backends
  already support on that path

Do not implement SharedSandbox, product UX, or a second graph executor.

## Acceptance Criteria

- an offline render-plane plugin stage is driven from the host assembly
- the processor came from `prepare_plugin_processor`
- v1 render-plane entry points vs deferred are written down in the batch log
  and, if needed, the integration map
- focused tests cover the offline path

## Validation

- targeted tests for the new offline host/render path
- `effigy qa:docs` if the integration map changes

## Evidence Required

- batch log
- validation actually run

## Stop Conditions

- the factory from card 001 cannot supply a live processor
- wiring requires a new render-plane API beyond existing plugin-stage handles
- scope expands into live audio-thread host pumping or Pulse workflow

## Next Task

If this card closes cleanly, auto-start
`docs/roadmaps/g11/batch-cards/003-g11-001-host-edge-proof-and-closeout.md`.
Mark `ready` only after card 001 closeout confirms the factory signature landed.
