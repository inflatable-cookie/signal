# 048 - g09.015 Local Scan Containment And Visibility

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Make the default interactive plugin browser surface show local launch targets
reliably when valid local plugins exist, without regressing back to inferred
server-only local buttons.

## Why This Batch Exists

The browser is now honest about local launchability, but the default interactive
entrypoint still commonly shows only server targets on real machines because the
local scan surface can stall or fail on broad system roots. That leaves the
operator-visible local posture too fragile for the main `effigy` entrypoint.

## Scope

- contain local scan work so one problematic installed plugin does not suppress
  local visibility for the entire browser surface
- keep local launch buttons tied to genuinely local scan truth
- improve operator feedback when local scan is partially degraded instead of
  collapsing to “no local buttons” without useful explanation
- keep the bounded proof path stable while the interactive path gets stronger

## Out Of Scope

- embedded plugin editor UI
- persistent interactive plugin sessions
- broad product-style browser UX redesign

## Acceptance Criteria

- the default `effigy demo:plugin-capability-browser` surface can still show
  local launch buttons when at least one scanned local plugin is healthy, even
  if another installed plugin misbehaves
- degraded local scan results remain explicit in the browser surface
- the noninteractive proof task remains stable and green

## Validation

- `effigy demo:plugin-capability-browser:proof`
- targeted interactive browser runs against multi-plugin system roots on the
  current machine
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- browser receipt and operator notes that reflect contained local scan posture
- updated manifest/scenario wording if the interactive/default split changes
- batch log with the actual local-scan outcomes and validation run

## Stop Conditions

- real local-scan containment requires a deeper host-side isolation design not
  already covered by the current contracts
- the interactive default task still cannot stay low-dependency while handling
  problematic installed plugins honestly

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is deeper live plugin interaction, broader plugin-browser
operator posture, or a planning pause.
