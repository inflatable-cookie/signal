# 003 - g11.001 Host-Edge Proof And Closeout

Status: blocked
Owner: core-product
Updated: 2026-08-17
Master spec refs: none (baseline-routed; no active strict spec)
Roadmap refs: g11.001
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md, docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md, docs/architecture/production-host-assembly-integration.md, docs/roadmaps/g11/001-production-host-assembly-wiring.md
Auto-start next card: no
Depends on: 002-g11-001-render-plane-consumer-wiring.md

## Objective

Prove the host-assembly path on a public `signal-host-local` host-edge test,
refresh front doors, and close `g11.001`.

## Scope

- extend or add public host-edge tests that exercise real bridge backends, not
  broker metadata-only sessions
- run `effigy validate` and record it
- refresh `LocalRuntimeHost` docs, architecture inventory, and `g11` front doors
- name the product-pull gate for `g11.002` without opening that milestone

## Acceptance Criteria

- public host-edge proof exists for the same path card 002 used
- `g11.001` goals are checked or explicitly deferred
- no remaining doc claims that plugin hosting is missing or discovery-only
- `g11/README.md` Next Task points at `g11.002` (deferred) or an operator
  planning checkpoint, not at a finished batch

## Validation

- `effigy validate`
- `effigy qa:docs`

## Evidence Required

- batch log
- validation actually run
- milestone closeout on `docs/roadmaps/g11/001-production-host-assembly-wiring.md`
  and `docs/roadmaps/g11/README.md`

## Stop Conditions

- proof still depends on metadata-only broker sessions
- closeout would reopen SharedSandbox implementation
- `effigy validate` fails in a way that changes the plan

## Next Task

Stop for operator review of the PR. Do not start `g11.002`.
