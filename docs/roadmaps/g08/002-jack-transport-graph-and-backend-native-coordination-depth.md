# 002 - JACK Transport, Graph, And Backend-Native Coordination Depth

Status: complete
Owner: core-product
Created: 2026-03-19
Depends on: g08.001
Vision tags: `LINUX`, `JACK`, `TRANSPORT`

## Problem

`g08.001` closes the bounded live Linux session-ownership seam, but JACK
transport, graph attachment, and backend-native coordination detail are still
outside shared runtime meaning. Without a bounded JACK contract, later Linux
session and workflow depth will fall back into backend-private callback and
graph policy again.

## Goals

- [ ] freeze runtime-owned JACK transport and graph coordination meaning
- [ ] expose one bounded JACK-native coordination substrate through shared runtime
- [ ] keep backend-native callback and daemon detail additive rather than authoritative

## Non-Goals

- [ ] no exhaustive JACK session-manager or distro-policy support matrix here
- [ ] no product-local transport UI or graph inspector UX

## Execution Plan

### Batch 2.1 - JACK Coordination Contract

- [x] freeze runtime-owned JACK transport, graph, and backend-native coordination meaning
- [x] define shared runtime versus JACK-private authority explicitly

### Batch 2.2 - Runtime JACK Baseline

- [x] materialize the first runtime-owned JACK transport and graph coordination receipts
- [x] align server-host export with the same JACK-native coordination model

### Batch 2.3 - Consumer Proof

- [x] prove the widened JACK transport and graph seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] JACK transport and graph coordination are runtime-owned and inspectable
- [x] backend-native callback and graph detail stay bounded and typed
- [x] later Linux ownership work can build on one explicit JACK authority line

## Risks And Mitigations

- Risk: JACK graph and transport meaning drifts into host-private callback or daemon policy.
- Mitigation: freeze one runtime-owned authority chain before widening runtime realization.

## Evidence Requirements

- [ ] log each meaningful tranche
- [ ] run focused validation after the runtime baseline lands
- [ ] record the next milestone step explicitly

## Batch 2.1 Outcome

Batch 2.1 freezes the bounded JACK coordination seam in
`docs/contracts/053-jack-transport-graph-and-backend-native-coordination-contract.md`.
That contract layers JACK transport posture, graph attachment, client role, and
guarded backend-native coordination on top of the closed live Linux ownership
boundary instead of inventing a competing JACK-only session shell.

It now makes the authority line explicit:

- live session ownership stays anchored in the closed `052` contract
- transport and graph coordination must compose through shared runtime,
  hardware, and supervision receipts instead of host-private callback policy
- JACK callback-thread, daemon, port-ID, and session-manager details remain
  private until later promotion
- Batch 2.2 now has one bounded contract target for runtime-owned JACK
  transport and graph receipts before public proof widens in Batch 2.3

## Batch 2.2 Outcome

Batch 2.2 makes the first JACK coordination seam real on shared runtime and
stable host-edge surfaces.

- `signal-runtime` now owns `RuntimeJackCoordinationSnapshot` plus typed
  transport-posture, graph-state, client-role, and guarded-coordination
  receipts derived from shared live Linux host-I/O and transport-session
  evidence
- the derivation path stays runtime-owned and bounded: non-JACK hosts export
  explicit `NotJack`, while JACK baselines compose through the closed live
  Linux ownership seam instead of backend-private callback policy
- `signal-host-local` now exports that same shared runtime answer as an
  explicit non-JACK baseline, and `signal-host-server` exports a bounded
  simulated JACK graph baseline on the same seam rather than rebuilding
  host-private JACK coordination truth
- Batch 2.3 is now the remaining work for this milestone: prove the widened
  JACK seam through public runtime, supervisor-tools, and stable host-edge
  consumer surfaces

## Batch 2.3 Outcome

Batch 2.3 closes the bounded JACK coordination proof seam through shared
runtime, supervisor-tools, and both stable host edges.

- public runtime now proves transport posture, graph coordination, client
  role, and guarded state through one downstream-style observation and
  supervisor seam
- stable local host edge now proves unsupported hosts answer this seam
  explicitly as `NotJack`
- stable server host edge now proves the bounded guarded JACK graph baseline
  stays runtime-owned instead of server-local graph policy
- supervisor-tools and Effigy now expose `signal.runtime.jack-coordination-boundary`
  plus `acceptance:jack-coordination-boundary` as the repo-owned consumer
  proof surface before `g08` widens into broader PipeWire and ALSA parity

## Next Task

Continue `g08.003` with Batch 3.1 by freezing runtime-owned PipeWire and ALSA
session-role, device-claim, and stream-policy parity meaning on top of the
closed live Linux ownership and JACK coordination seams.
