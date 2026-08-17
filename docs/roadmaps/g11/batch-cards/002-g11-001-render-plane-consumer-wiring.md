# 002 - g11.001 Render-Plane Consumer Wiring

Status: complete
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

Closed. One offline proof path: host prepares an in-process CLAP processor, a
render-plane Sum stage uses that handle, `render_plan_to_pcm` processes audio.

## Acceptance Criteria

- [x] an offline render-plane plugin stage is driven from the host assembly
- [x] the processor came from `prepare_plugin_processor`
- [x] v1 render-plane entry points vs deferred are written down in the batch log
  and, if needed, the integration map
- [x] focused tests cover the offline path

## Validation

- `cargo test -p signal-host-local --test public_host_edge_plugin_processor`

## Evidence Required

- batch log: `docs/logs/2026-08/17-g11-001-batch-1-3-render-plane-consumer-wiring.md`

## Stop Conditions

None fired.

## Next Task

Execute
`docs/roadmaps/g11/batch-cards/003-g11-001-host-edge-proof-and-closeout.md`.
