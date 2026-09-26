# Retired backlog: post-g10 rebuild-on-demand candidates

Status: triage (non-authoritative)
Retired from: `docs/roadmaps/backlog/post-g10-rebuild-on-demand.md` (roadmap-backlog surface retired 2026-09-09)
Source refs: backlog item created 2026-06-11, updated 2026-08-17; `docs/roadmaps/archive/g10.md`; Contract `072`; `docs/roadmaps/strategic-runway.md` (Horizons B–C)
Owner: core-product

Triage holds this as unresolved/deferred candidacy only. It is not execution
authority; the active roadmap and contract front doors remain authoritative.
Promotion is not approval until an active generation task owns it.

## Deferred candidates (product-pull only)

Each item below is pulled into a future generation only when a Loophole
product feature needs it — never speculatively.

- **Engine server / out-of-process engine.** Nothing was kept from
  `signal-host-server` (a print-and-exit clone). If wanted: design transport
  plus session dispatch fresh on `signal-ipc` shared memory.
- **Device handling depth.** Device-change notifications, input/duplex streams
  for recording, explicit device-selection UI contract. Builds on `g10.003`
  cpal enumeration. Open gate: `g10.017` recording capture and live monitoring
  landed; hardware alignment and consumer evidence remain an explicit operator
  gate (see `docs/roadmaps/archive/g10.md`).
- **Resampling/time domain.** Higher-quality SRC tiers beyond the `g10.008`
  polyphase table.
- **Beat tracking upgrade.** Replace fixed-grid beat placement with DP/HMM
  tracking for drifting tempo; widen the 70–180 BPM default range. Builds on
  the rhythm core kept by `g10.006`.
- **Graph successor.** Production node-graph execution around the render
  plane's control/render split (topological ordering, PDC via delay insertion,
  retained stage state, preallocated buffers). Nothing from `signal-graph`'s
  execution path is reusable; its telemetry vocabulary may inform the control
  side.
- **Multichannel/loudness breadth.** Surround channel weights (1.41 Ls/Rs) and
  true multichannel metering when Loophole grows beyond stereo.
- **Product browser / workflow shells.** Inventory UX and downstream-app
  workflow remain outside Signal unless explicitly promoted.

## Deliberately not carried over (shipped or closed)

- Real plugin hosting for CLAP, VST3, AU, and LV2 (discovery, lifecycle,
  `process()`, events, state, GUI paths; `signal-plugin-sandbox` broker;
  `signal-plugin-bridge` placement-agnostic backends; offline render-plane
  integration). Shipped through `g09` and later hosting batches; owned by
  Contract `072` and the architecture docs. Do not reopen.
- Production host-assembly wiring — closed in `g11.001`.
- SharedSandbox tier — closed in `g11.002` (map at
  `docs/knowledge/architecture/shared-sandbox-multiplexing.md`).

## Promotion condition

Promote one item only when the operator selects it in
`docs/roadmaps/strategic-runway.md` or an active generation task requires it.
Migration from the retired backlog is not approval.

## Open questions

- Which product-pulled Horizon B item opens next (graph successor, device
  depth, or consumer-release depth).
- Whether engine-server returns before product pull justifies it.
- The `g10.017` hardware-alignment and consumer-evidence gate.

## Next check

Review each candidate during the next refresh or docs-currentness pass and
promote, merge, or remove it.
