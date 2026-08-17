# g11 Milestones

Status: active generation opening; `g10` stretch audit complete; no implementation
batch started
Updated: 2026-08-17

## At a glance

- `g11` opens the post-stretch integration generation: turn shipped plugin-hosting
  substrate into a trustworthy Pulse-facing production path.
- Plugin hosting baseline work is **not** in scope — CLAP, VST3, AU, and LV2
  hosting already ship through adapter crates, `signal-plugin-sandbox`, and
  `signal-plugin-bridge`. See Contract `072`.
- First milestone: **`g11.001` production host-assembly wiring** — Batch 1.1
  complete; Batch 1.2 ready
- Second milestone: **`g11.002` SharedSandbox tier** — deferred until `g11.001`
  closes and product pull exists. Contract `014` already owns semantics; this is
  implementation, not a research program.
- Formal `g10` generation closeout may run in parallel as a docs-only hygiene
  batch; it does not block `g11.001` Batch 1.1.

## Why this generation matters now

`g10` finished audit remediation and the stretch audit. Signal's reusable runtime,
DSP, analysis, graph, and plugin-hosting crates are live. The remaining gap is
**integration depth**: consumers still cannot rely on `signal-host-local` as the
production path from discovery through bridge backends to render-plane execution.

## Generation runway

1. **`g11.001`** — production host-assembly wiring (active)
2. **`g11.002`** — SharedSandbox tier (deferred; contract-backed)
3. **Backlog pulls** — graph successor, device depth, analysis breadth only when
   Loophole or another consumer names the dependency

Do not reopen stretch, Automatic, or RealtimePreview adoption work from this
generation.

## Milestone Map

- `g11.001` `active`
  - production host-assembly wiring; Batch 1.1 complete; Batch 1.2 ready
- `g11.002` `deferred`
  - SharedSandbox tier; depends on `g11.001` closeout and product pull

## Working Rule

- treat Contract `072` as the hosting baseline authority
- treat Contract `014` as the SharedSandbox semantics authority
- keep `signal-host-local` a thin wrapper over runtime-owned meaning (Contract
  `009`)
- do not infer Loophole UI, Chorus mixer, or downstream workflow scope

## Next Task

Execute
[`001-g11-001-bridge-backend-factory.md`](./batch-cards/001-g11-001-bridge-backend-factory.md).
Authority: `docs/architecture/production-host-assembly-integration.md`.
