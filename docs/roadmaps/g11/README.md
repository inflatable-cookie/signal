# g11 Milestones

Status: `g11.001` complete; `g11.002` deferred pending product pull
Updated: 2026-08-17

## At a glance

- `g11` is the post-stretch integration generation: turn shipped plugin-hosting
  substrate into a trustworthy Pulse-facing production path.
- Plugin hosting baseline work is **not** in scope — CLAP, VST3, AU, and LV2
  hosting already ship through adapter crates, `signal-plugin-sandbox`, and
  `signal-plugin-bridge`. See Contract `072`.
- First milestone: **`g11.001` production host-assembly wiring** — complete.
  `LocalRuntimeHost::prepare_plugin_processor` constructs bridge backends;
  one offline render-plane path is proven on a public host-edge test.
- Second milestone: **`g11.002` SharedSandbox tier** — deferred until product
  pull exists. Contract `014` already owns semantics; this is implementation,
  not a research program.
- Formal `g10` generation closeout may run in parallel as a docs-only hygiene
  batch; it does not reopen `g11.001`.

## Why this generation matters now

`g10` finished audit remediation and the stretch audit. Signal's reusable runtime,
DSP, analysis, graph, and plugin-hosting crates are live. `g11.001` closed the
host-assembly gap from scan → placement → bridge backend → offline render-plane
execution. Remaining generation work is SharedSandbox multiplexing when a
consumer pulls it.

## Generation runway

1. **`g11.001`** — production host-assembly wiring (complete)
2. **`g11.002`** — SharedSandbox tier (deferred; contract-backed)
3. **Backlog pulls** — graph successor, device depth, analysis breadth only when
   Loophole or another consumer names the dependency

Do not reopen stretch, Automatic, or RealtimePreview adoption work from this
generation.

## Milestone Map

- `g11.001` `complete`
  - production host-assembly wiring; Batches 1.1–1.4 closed
- `g11.002` `deferred`
  - SharedSandbox tier; depends on product pull. Contract `014` owns semantics.

## Working Rule

- treat Contract `072` as the hosting baseline authority
- treat Contract `014` as the SharedSandbox semantics authority
- keep `signal-host-local` a thin wrapper over runtime-owned meaning (Contract
  `009`)
- do not infer Loophole UI, Chorus mixer, or downstream workflow scope

## Next Task

Stop for operator review of the `g11.001` PR. Do not start `g11.002` until
product pull exists. Authority: Contract `014` and
`docs/roadmaps/g11/002-shared-sandbox-tier.md`.
