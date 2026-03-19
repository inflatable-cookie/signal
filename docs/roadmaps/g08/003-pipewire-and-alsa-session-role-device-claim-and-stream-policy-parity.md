# 003 - PipeWire And ALSA Session-Role, Device-Claim, And Stream-Policy Parity

Status: active
Owner: core-product
Created: 2026-03-19
Depends on: g08.002
Vision tags: `LINUX`, `PIPEWIRE`, `ALSA`

## Problem

`g08.001` closed the bounded live Linux ownership seam and `g08.002` closed
the first JACK-native coordination seam, but PipeWire and ALSA still do not
share one explicit runtime-owned answer for session role, device-claim
posture, and stream-policy parity. Without that boundary, later Linux device,
preview, and workflow work will drift back into backend-private or host-local
policy.

## Goals

- [ ] freeze runtime-owned PipeWire and ALSA session-role, device-claim, and stream-policy parity meaning
- [ ] expose one bounded parity substrate across shared runtime and stable host edges
- [ ] keep backend-native daemon, node, and stream detail additive rather than authoritative

## Non-Goals

- [ ] no exhaustive PipeWire graph-policy or ALSA distro-policy matrix here
- [ ] no product-local device browser, session UI, or repair UX

## Execution Plan

### Batch 3.1 - PipeWire And ALSA Parity Contract

- [ ] freeze runtime-owned PipeWire and ALSA session-role, device-claim, and stream-policy parity meaning
- [ ] define shared runtime versus backend-native authority explicitly

### Batch 3.2 - Runtime PipeWire And ALSA Baseline

- [ ] materialize the first runtime-owned PipeWire and ALSA parity receipts
- [ ] align stable host-edge export with the same parity model

### Batch 3.3 - Consumer Proof

- [ ] prove the widened PipeWire and ALSA parity seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [ ] PipeWire and ALSA session-role, device-claim, and stream-policy parity are runtime-owned and inspectable
- [ ] backend-native daemon or stream detail stays bounded and typed
- [ ] later Linux workflow and acceptance work can build on one explicit PipeWire and ALSA authority line

## Risks And Mitigations

- Risk: PipeWire and ALSA stream-policy truth drifts into host-private daemon or stream wrappers.
- Mitigation: freeze one runtime-owned parity contract before widening runtime realization.

## Evidence Requirements

- [ ] log each meaningful tranche
- [ ] run focused validation after the runtime baseline lands
- [ ] record the next milestone step explicitly

## Next Task

Continue `g08.003` with Batch 3.1 by freezing runtime-owned PipeWire and ALSA
session-role, device-claim, and stream-policy parity meaning on top of the
closed live Linux ownership and JACK coordination seams.
