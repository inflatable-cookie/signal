# g08 Milestones

Status: active
Updated: 2026-03-21

## Why this generation matters now

`g07` closed the bounded feature-expansion queue: routing, Linux portability,
controller substrate, and sample-domain transform services are now typed,
proved, and grouped through one reusable closeout gate. The next Signal-owned
bottleneck is no longer whether those seams exist. It is whether Signal can
turn the deferred live-ownership and richer workflow fronts into shared runtime
substrate without collapsing back into host-local or product-local policy.

This generation therefore focuses on the next reusable depth:

- live Linux audio backend ownership across ALSA, JACK, and PipeWire session
  lifecycle instead of bounded availability or fallback receipts only
- richer plugin protocol depth such as LV2 worker/URID/patch and complex
  runtime-owned bus or pin negotiation beyond the current bounded baseline
- immersive routing, object rendering, room policy, and deployment breadth on
  top of the closed multichannel and spatial substrate
- vendor-protocol control-surface and advanced-device feedback depth without
  promoting product-local controller UX into shared runtime
- preview-device, audition, and workflow-adjacent media services that remain
  runtime-owned instead of browser-local or editor-local glue

## Dependency order

1. freeze live Linux backend ownership and session lifecycle first
2. deepen LV2 and complex plugin protocol breadth on the now-explicit Linux and
   routing substrate
3. widen immersive routing and room-policy meaning after live ownership is
   clearer
4. deepen richer external-device protocol and feedback services after endpoint
   and controller substrate are already shared
5. move preview-device and workflow-adjacent media services forward only where
   they stay runtime-owned
6. close with integrated acceptance and a generation closeout gate again

## Milestone map

- `g08.001` `complete`
  - live Linux audio backend ownership and session lifecycle substrate
- `g08.002` `complete`
  - JACK transport, graph, and backend-native coordination depth
- `g08.003` `complete`
  - PipeWire and ALSA session-role, device-claim, and stream-policy parity
- `g08.004` `complete`
  - LV2 worker, URID, patch, and extension negotiation baseline
- `g08.005` `complete`
  - complex plugin pin-matrix and dynamic bus negotiation depth
- `g08.006` `complete`
  - immersive object rendering and room-policy substrate
- `g08.007` `complete`
  - speaker deployment, fold-down, and monitoring scene depth
- `g08.008` `complete`
  - renderer-capability negotiation and immersive export baseline
- `g08.009` `complete`
  - advanced control-surface display, motor, and haptic transport
- `g08.010` `complete`
  - control-surface scene mapping, feedback pages, and safe action graphs
- `g08.011` `active`
  - preview-output routing, audition sink ownership, and low-latency device policy
- `g08.012` `todo`
  - preview-browser queue, media audition, and transform scheduling depth
- `g08.013` `todo`
  - asset/session transform persistence, retention, and cache placement policy
- `g08.014` `todo`
  - live external MIDI device ownership and backend parity depth
- `g08.015` `todo`
  - cross-backend device protocol and live workflow acceptance
- `g08.016` `todo`
  - Linux live backend acceptance and failure-injection depth
- `g08.017` `todo`
  - immersive render and monitoring acceptance depth
- `g08.018` `todo`
  - control-surface and preview workflow acceptance depth
- `g08.019` `todo`
  - integrated live-ownership and workflow acceptance depth
- `g08.020` `todo`
  - generation closeout and downstream workflow readiness gate

## Lane structure

### Lane A - Linux Live Ownership And Plugin Protocols

`001 -> 002 -> 003 -> 004 -> 005`

Turn bounded Linux and plugin parity receipts into runtime-owned live session,
graph, and protocol depth.

### Lane B - Immersive Routing And Room Deployment

`006 -> 007 -> 008`

Extend the closed spatial substrate into richer object, room, monitoring, and
deployment semantics without drifting into product-local immersive UX.

### Lane C - Device Protocol And Feedback Workflows

`009 -> 010 -> 014 -> 015`

Deepen advanced-device and control-surface protocol breadth while keeping
device meaning inside typed shared runtime surfaces.

### Lane D - Preview And Transform Workflow Services

`011 -> 012 -> 013`

Move workflow-adjacent preview, audition, and transform services forward only
where they remain runtime-owned and inspectable.

### Lane E - Acceptance And Closeout

`016 -> 017 -> 018 -> 019 -> 020`

Prove the widened live-ownership and workflow substrate before claiming the
generation is ready for downstream reliance.

## Working rules for this thread

- keep live backend, routing, device, and workflow semantics inside Signal
  runtime, plugin, and hardware crates
- do not promote product-local browser, editor, controller-page, or immersive
  console UX into this generation
- prefer one runtime-owned truth per seam over backend-native capability tables
  or host-local workaround layers
- keep `g08` as the single active queue and move anything not generation-
  critical back into backlog instead of reopening `g07`

## Next Task

Continue `g08.011` with Batch 11.2 by materializing the first runtime-owned
preview-output routing, audition-sink ownership, and low-latency device-policy
receipts, then align stable host-edge export to the same bounded model.
