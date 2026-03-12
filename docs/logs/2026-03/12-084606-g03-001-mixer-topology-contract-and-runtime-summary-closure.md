# 2026-03-12 08:46:06 Europe/London - g03.001 mixer topology contract and runtime summary closure

Closed `g03.001` by making routed mixer ownership explicit from
`signal-graph` contracts through `signal-runtime` observation summaries, then
pinning the first graph/runtime proofs around that seam.

This closeout matters because later metering, automation, warp, and render work
can now depend on one typed mixer-topology contract instead of rediscovering
track, bus, send/return, and console ownership from loosely related fields.

Milestone-close evidence:

- `signal-graph` topology metadata is now explicit about:
  - `track_lane_id`
  - `bus_group_id`
  - `console_group_id`
  - `send_return_id`
- graph contract validation now rejects topology-role nodes that omit the ids
  their mixer role requires
- `signal-runtime` projections and planned-node snapshots now carry the same
  explicit ownership ids instead of overloading one generic lane/bus field
- `RuntimeExecutionTopologySummary` now exports reusable routed mixer summaries
  for:
  - track lanes
  - bus groups
  - console groups
  - send/return routes
- runtime meter-source metadata now carries the same explicit ownership ids so
  `g03.002` can build routed metering on the same vocabulary
- `signal-host-local` demo graph contracts were updated to the explicit
  topology projection shape so host-local topology assembly stays aligned with
  the runtime contract

Focused proof coverage:

- `signal-graph`
  - contract summaries now prove missing explicit ids are surfaced as
    deterministic issues
  - routed fixtures still cover direct, fan-in, fan-out, and send/return
    execution behavior
- `signal-runtime`
  - scheduler topology summaries now count track, bus, send/return, and
    console groups against the explicit ownership fields
  - projected graph contracts now materialize reusable track-lane, bus-group,
    and console-group summaries in runtime observations
  - send/return projections now materialize one explicit routed summary with
    separate send-node and return-node membership plus input/output bus traces

Deferred gaps at close:

- no device-chain latency compensation or final mix-delay alignment yet
- no workflow/session editing semantics are introduced by this milestone
- no routed metering or loudness policy is attached to the new mixer summary
  yet; that opens in `g03.002`

Validation:

- `cargo fmt --all`
- `cargo test -p signal-graph`
- `cargo test -p signal-runtime`
- `git diff --check`

Validation note:

- `cargo test -p signal-host-local` currently fails before exercising this
  tranche because `LocalRuntimeHost` is missing several newer
  `RuntimeSupervisorApi` trait items (`start_recording_capture`,
  `finish_recording_capture`, `cancel_recording_capture`,
  `reconcile_media_assets`, `reconcile_warp_clips`). That failure is outside
  the routed mixer topology batch and did not block the focused `g03.001`
  graph/runtime validation surface.

Next task:

Open `g03.002` and thread routed metering, loudness-oriented summaries, and
diagnostics export through the now-explicit mixer-topology contract.
