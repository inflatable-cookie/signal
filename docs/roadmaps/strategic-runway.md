# Signal Strategic Runway

Status: active planning surface
Owner: core-product
Updated: 2026-08-17
Vision refs: `docs/vision/001-signal-vision.md`
Depends on: `docs/roadmaps/generation-index.md`, `docs/roadmaps/backlog/post-g10-rebuild-on-demand.md`, `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`

## Purpose

Shape Signal's long-horizon direction after the `g10` stretch audit without
pretending every future batch is already known. This file connects vision,
architecture, contracts, and generation sequencing. It is not a ready card and
does not authorize execution.

## Strategic direction

Signal remains one reusable audio stack for Loophole, Finch, and future apps.
The long-horizon outcome is unchanged: write DSP, analysis, graph execution,
and runtime coordination once; consume them across products without duplicating
core signal-processing work.

Current constraints that outlast the next batch:

- keep reusable DSP, analysis, and graph logic inside Signal-owned crates
- keep plugin SDK glue and hardware shims thin and replaceable
- preserve real-time safety on engine/runtime paths
- treat untrusted plugin code as the default out-of-process isolation target
- pull deferred capability only when a product feature needs it, not
  speculatively

## Current shape

Evidence as of 2026-08-17:

- `g10` is the active generation. Realtime render, DSP, analysis, graph, and
  runtime crates are live and regression-tested.
- The stretch audit (`g10.036`–`g10.042`) is complete. Transparent, Dream, and
  Cyclic are explicit admitted characters. Automatic is closed.
  `RealtimePreview` is proven and deliberately unadopted.
- Signal is baseline-routed with no active strict lane and no ready batch.
- CLAP, VST3, AU, and LV2 hosting is implemented through adapter crates,
  `signal-plugin-sandbox`, and `signal-plugin-bridge`. The remaining plugin
  seam is integration depth — SharedSandbox tier and production host-assembly
  wiring through `signal-host-local` — not "build hosting from scratch."
- Deferred rebuild candidates live in backlog, not in the active queue.

Material contradictions resolved in this refresh:

- roadmap and architecture front doors now agree that plugin hosting baseline
  work is shipped
- stale post-demolition backlog language is marked superseded where needed
- long-horizon sequencing is recorded here instead of being inferred from stale
  batch pointers

## Horizon model

### Horizon A — close `g10`, choose the next Signal-only lane

Outcome: the current generation is honestly closed and one bounded Signal-only
target becomes the next executable lane.

Depends on:

- operator selection of the next Signal-owned target
- agreement that no stretch-audit or Automatic work remains open
- roadmap front doors and backlog posture staying aligned

Unlocks: a fresh strict lane or baseline-routed milestone under `g11` without
carrying stale `g10` pointers forward.

Excludes: Loophole UI/workflow ownership, Chorus mixer realization, and any
Automatic or RealtimePreview adoption work.

Review trigger: operator names a target or rejects all listed candidates.

### Horizon B — product-pulled integration and runtime depth

Outcome: Signal's shipped substrate becomes a trustworthy production path for
Loophole and other consumers without reopening fake scaffold work.

Primary bets, in product-pull order:

1. **Production host-assembly wiring** — closed in `g11.001`
2. **SharedSandbox tier** — one broker process hosting many plugins; Batch 2.1
   ready (`docs/architecture/shared-sandbox-multiplexing.md`)
3. **Graph successor** — production node-graph execution around the render
   plane's control/render split: topological ordering, PDC via delay insertion,
   retained stage state, preallocated buffers
4. **Device handling depth** — device-change notifications, input/duplex
   streams, explicit device-selection contract on top of `g10.003` enumeration
5. **Consumer release depth** — source-consumer gates, publication promotion,
   and backend breadth already deferred from earlier generations

Unlocks: Loophole can consume Signal's already-shipped hosting/runtime
mechanism through a production assembly instead of test-only proof paths.

Excludes: rebuilding CLAP/VST3/AU/LV2 hosting from scratch; speculative
engine-server, beat-tracking, or multichannel work without product pull.

Review trigger: Loophole names a blocking product dependency or Signal proves a
consumer path still relies on demo/test wiring only.

### Horizon C — analysis and substrate breadth

Outcome: richer reusable analysis and resampling substrate without reopening
closed stretch programs.

Bets:

- beat-tracking upgrade beyond fixed-grid placement
- higher-quality SRC tiers beyond the `g10.008` polyphase table
- multichannel/loudness breadth when Loophole grows beyond stereo

Unlocks: downstream apps can depend on deeper analysis without forking local
copies.

Excludes: any transparent or creative stretch successor lane; those programs
are closed on the frozen baseline.

Review trigger: repeated consumer demand or a named analysis contract gap.

### Horizon D — ecosystem consolidation

Outcome: the temporary C++ compatibility island shrinks as Rust-owned runtime
components replace it incrementally.

Depends on:

- trustworthy production integration from Horizon B
- stable consumer contracts from earlier release gates
- explicit migration batches rather than silent shims

Unlocks: one durable runtime story across Finch and Loophole with fewer
compatibility islands.

Excludes: big-bang rewrite or compatibility shims unless a contract explicitly
authorizes them.

Review trigger: a migration batch would touch trust edges or break an existing
consumer contract.

## Strategic bets and trade-offs

| Bet | Why it matters | Main dependency | Non-goal |
| --- | --- | --- | --- |
| Host-assembly wiring | turns shipped hosting into a consumer path | `signal-host-local` + bridge | rebuilding adapters from scratch |
| Graph successor | unlocks real production graphs, not demos | honest render/control split | reviving `signal-graph` execution path |
| Rebuild-on-demand queue | prevents speculative fake depth | operator/product trigger | scheduling backlog items without pull |
| Frozen stretch baseline | protects shipped product truth | none | Automatic routing or RealtimePreview adoption |
| Incremental C++ replacement | matches vision delivery envelope | production integration proof | silent topology shims |

Accepted uncertainty:

- whether the first post-`g10` lane should be host-assembly wiring, graph
  successor, or consumer-release depth
- whether `g10` should roll to `g11` immediately or stay open for one more
  bounded Signal-only closeout card
- whether SharedSandbox or engine-server returns before product pull justifies it
  (SharedSandbox pulled 2026-08-17; engine-server still backlog)

## Runway

Meaningful milestone transitions, not a task queue:

1. **Now:** execute `g11.002` Batch 2.1 (broker multiplexing).
2. **After `g11.002`:** next product-pulled Horizon B item (graph successor or
   device depth), not a speculative lane.
3. **First `g11` tranche:** host-assembly (`g11.001`, complete) then SharedSandbox
   (`g11.002`, in flight).
4. **Second `g11` tranche:** pull the next product-pulled item from backlog only
   after the current tranche closes with evidence.
5. **Mid horizon:** promote analysis/substrate breadth (Horizon C) when a named
   contract or consumer need exists.
6. **Long horizon:** migration/consolidation batches (Horizon D) once production
   integration is trustworthy.

Recommended default if the operator wants a concrete starting point:

- execute `g11.002` Batch 2.1 (broker multiplexing) from
  `docs/architecture/shared-sandbox-multiplexing.md`

## Promotion map

| Outcome | Canonical destination |
| --- | --- |
| long-horizon outcomes and constraints | `docs/vision/001-signal-vision.md` |
| system shape and invariants | `docs/architecture/` |
| durable authority or behaviour | `docs/contracts/` |
| deferred rebuild candidates | `docs/roadmaps/backlog/post-g10-rebuild-on-demand.md` |
| generation sequencing and rollover | `docs/roadmaps/generation-index.md` |
| executable batches | `docs/roadmaps/g11/` milestones and cards |
| refresh and atlas evidence | `docs/logs/2026-08/17-northstar-refresh-and-atlas-runway.md` |

## Open operator decisions

1. Does formal `g10` generation closeout need a separate docs-only card?

`g11.002` product pull landed 2026-08-17 (operator). v1 grouping is plugin
type identity. Batch 2.0 closed.

## Next Task

Execute `docs/roadmaps/g11/batch-cards/005-g11-002-broker-multiplexing.md`.
