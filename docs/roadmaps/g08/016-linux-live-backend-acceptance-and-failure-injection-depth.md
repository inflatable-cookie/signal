# 016 - Linux Live Backend Acceptance And Failure-Injection Depth

Status: complete
Owner: core-product
Created: 2026-03-22
Depends on: g08.015
Vision tags: `LINUX`, `LIVE`, `ACCEPTANCE`

## Problem

`g08.015` closes the bounded device-protocol acceptance seam, but the shared
consumer proof for live Linux backend ownership, guarded recovery, and
failure-injection depth is still spread across earlier Linux live-ownership,
JACK, PipeWire/ALSA parity, and backend clock-topology boundaries.

Without one explicit acceptance milestone here, later Linux depth risks
drifting into daemon-local policy, backend-specific recovery scripts, or ad
hoc failure reruns that shared consumers cannot rely on.

## Goals

- [ ] freeze one shared acceptance target for live Linux backend ownership and
      failure-injection depth
- [ ] keep the acceptance seam grounded in existing runtime-owned Linux live
      ownership, JACK coordination, PipeWire/ALSA parity, and clock-topology
      receipts
- [ ] avoid backend-local recovery policy or daemon-local session glue becoming
      the shared proof surface

## Non-Goals

- [ ] no distro certification matrix or daemon-specific troubleshooting guide
- [ ] no product-local rehearsal console, device browser, or patchbay shell as
      the shared acceptance surface

## Execution Plan

### Batch 16.1 - Linux Live Acceptance Contract

- [x] freeze the shared live Linux backend acceptance and failure-injection
      contract
- [x] define the mandatory runtime, supervisor, and stable host-edge proof
      spine explicitly

### Batch 16.2 - Acceptance Descriptor And Task

- [x] wire the first repo-owned descriptor and acceptance lane for the shared
      Linux live-backend seam
- [x] keep optional backend-native recovery depth explicit rather than folding
      it into the mandatory shared contract

### Batch 16.3 - Consumer Proof Closure

- [x] prove the widened Linux live-backend acceptance seam through shared
      runtime, supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] live Linux backend ownership and failure-injection acceptance are
      repo-owned and inspectable
- [x] backend-local daemon or recovery detail stays bounded and typed
- [x] later immersive, preview, and integrated acceptance work can build on
      one explicit Linux live acceptance seam

## Risks And Mitigations

- Risk: Linux live acceptance drifts into backend-local daemon policy or
  product-specific recovery glue.
- Mitigation: freeze one shared acceptance contract before widening further
  failure-injection or integrated depth.

## Evidence Requirements

- [x] log each meaningful tranche
- [x] run focused validation after descriptor/task changes land
- [x] record the next milestone step explicitly

## Batch 16.1 Outcome

- `g08` now has a frozen shared Linux live-backend acceptance contract in
  `docs/contracts/067-live-linux-backend-acceptance-and-failure-injection-contract.md`
  instead of leaving grouped live Linux proof fragmented across the Linux live
  ownership, JACK coordination, PipeWire/ALSA parity, and clock-topology
  seams
- the shared acceptance lane is now required to compose through public
  runtime receipts, supervisor export, and both stable host edges rather than
  daemon-local policy or backend-specific recovery glue
- the grouped descriptor, Effigy acceptance lane, and broader advisory versus
  deferred Linux failure depth remain explicitly deferred until Batch 16.2
  and Batch 16.3

## Batch 16.2 Outcome

- `signal-supervisor-tools` now exposes one machine-readable
  `signal.runtime.linux-live-acceptance-lane` descriptor so the shared Linux
  live acceptance seam is inspectable without reading multiple isolated
  boundary descriptors by hand
- Effigy now owns one runnable `effigy acceptance:linux-live-acceptance-lane`
  task that composes the already-closed Linux live ownership, JACK
  coordination, PipeWire/ALSA parity, and clock-topology acceptance proofs
  into one bounded shared lane
- broader backend-native daemon and recovery depth remain explicitly advisory
  or deferred instead of being smuggled into the required path

## Batch 16.3 Outcome

- the shared Linux live acceptance lane now has one grouped consumer-facing
  supervisor export proof instead of only a grouped descriptor, so Linux live
  ownership, JACK coordination, PipeWire/ALSA parity, and clock-topology truth
  are proven consumable together on one shared path
- `effigy acceptance:linux-live-acceptance-lane` now composes the existing
  Linux boundary proofs, the grouped export proof, and the machine-readable
  descriptor into one reusable acceptance lane
- `g08.016` is now complete, and the next `g08` queue is immersive render and
  monitoring acceptance depth

## Completion

`g08.016` is complete. The bounded Linux live backend acceptance and
failure-injection seam is now frozen, grouped, proved through one shared
consumer path, and ready for later immersive and integrated acceptance work to
build on.

## Next Task

Continue `g08.017` with Batch 17.1 by freezing the shared immersive render and
monitoring acceptance contract on top of the closed immersive room-policy,
deployment-monitoring, renderer-export, and spatial consumer seams.
