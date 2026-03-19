# 001 - Live Linux Audio Backend Ownership And Session Lifecycle Substrate

Status: active
Owner: core-product
Created: 2026-03-19
Depends on: g07.020
Vision tags: `LINUX`, `BACKEND`, `LIVE-OWNERSHIP`

## Problem

`g07` proved bounded Linux backend identity, parity, and fallback meaning, but
Signal still does not own live ALSA, JACK, and PipeWire session lifecycle
truth. Without a runtime-owned live backend ownership seam, later routing,
device, preview, and controller workflows will fall back into backend-private
or host-local policy again.

## Goals

- [ ] freeze runtime-owned live Linux backend ownership and session lifecycle meaning
- [ ] expose one bounded live ownership substrate across ALSA, JACK, and PipeWire
- [ ] keep backend-native detail additive rather than the source of shared truth

## Non-Goals

- [ ] no exhaustive Linux distro or environment certification matrix here
- [ ] no product-local device picker or session UX

## Execution Plan

### Batch 1.1 - Ownership Contract

- [x] freeze the live Linux backend ownership and session lifecycle contract
- [x] define shared runtime versus backend-native authority explicitly

### Batch 1.2 - Runtime Baseline

- [x] materialize the first runtime-owned live backend ownership receipts
- [x] align server-host export with the same session-lifecycle model

### Batch 1.3 - Consumer Proof

- [x] prove the widened live Linux backend ownership seam through shared
      runtime, supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] live Linux backend ownership is runtime-owned and inspectable
- [ ] ALSA, JACK, and PipeWire session lifecycle meaning stays bounded and typed
- [x] later `g08` Linux and workflow work can build on one explicit authority line

## Risks And Mitigations

- Risk: backend-native detail leaks into the shared contract as ad hoc host policy.
- Mitigation: freeze one runtime-owned authority chain before widening runtime realization.

## Evidence Requirements

- [ ] log each meaningful tranche
- [ ] run focused validation after the runtime baseline lands
- [ ] record the next milestone step explicitly

## Batch 1.1 Outcome

Batch 1.1 freezes the live Linux backend ownership boundary in
`docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`.
That contract layers live ALSA, JACK, and PipeWire session ownership on top of
the closed `g07` portability and parity seams instead of inventing a separate
Linux-only session shell.

It now makes the ownership posture explicit:

- backend identity and parity remain anchored in the closed `040` and `041`
  contracts
- live attach, running, recovery, release, and unavailable posture must reuse
  the shared hardware, supervision, and external-I/O authority chain
- backend-native daemon, graph, transport, and session-manager details remain
  private until later promotion
- Batch 1.2 now has one bounded target for runtime-owned live backend
  ownership receipts instead of reopening Linux session authority during
  implementation

## Batch 1.2 Outcome

Batch 1.2 turns the frozen `g08.001` contract into a real runtime-owned receipt
family. `signal-runtime` now owns `RuntimeLinuxBackendSessionSnapshot` plus
typed ownership, lifecycle, device-claim, session-role, and guarded-fallback
states, derived from shared `RuntimeHostIoSummary` rather than backend-local
daemon or reconnect policy.

The baseline is intentionally bounded but meaningful:

- runtime observation and supervisor export now carry one machine-readable live
  Linux backend session seam even when the rest of the host/report surface is
  unchanged
- host-local now exports an explicit `NotLinux` answer instead of leaving this
  seam absent on non-Linux hosts
- server-host now publishes a simulated PipeWire guarded session baseline,
  proving the same runtime-owned DTO family can describe Linux live ownership
  without reopening host-private session taxonomy
- ALSA, JACK, and PipeWire ownership posture are covered in focused runtime
  classification tests before public consumer proof widens in Batch 1.3

## Batch 1.3 Outcome

Batch 1.3 closes the consumer seam for live Linux backend ownership through
shared runtime, both stable host edges, and a machine-readable supervisor-tools
descriptor. The runtime-owned `RuntimeLinuxBackendSessionSnapshot` is now
proven consumable without backend-private Linux session reconstruction.

The closed proof seam is intentionally bounded:

- public runtime now proves ALSA, JACK, and PipeWire ownership, lifecycle,
  device-claim, role, and guarded-fallback truth on one downstream-style
  surface
- local host edge proves non-Linux hosts answer this seam explicitly as
  `NotLinux` instead of omitting it
- server host edge proves the bounded PipeWire-style baseline stays runtime-
  owned instead of becoming server-local Linux policy
- supervisor-tools and Effigy now expose one repo-owned
  `linux-live-ownership-boundary` acceptance surface before `g08` moves into
  deeper JACK coordination work

## Next Task

Continue `g08.002` with Batch 2.2 by materializing the first runtime-owned
JACK transport, graph, client-role, and guarded-coordination receipt family
across runtime, supervision, and stable host-edge surfaces.
