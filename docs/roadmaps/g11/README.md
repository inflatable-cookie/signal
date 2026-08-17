# g11 Milestones

Status: `g11.001` complete; `g11.002` Batch 2.0 complete, Batch 2.1 ready
Updated: 2026-08-17

## At a glance

- `g11` is the post-stretch integration generation: turn shipped plugin-hosting
  substrate into a trustworthy Pulse-facing production path.
- Plugin hosting baseline work is **not** in scope — CLAP, VST3, AU, and LV2
  hosting already ship through adapter crates, `signal-plugin-sandbox`, and
  `signal-plugin-bridge`. See Contract `072`.
- First milestone: **`g11.001` production host-assembly wiring** — complete.
- Second milestone: **`g11.002` SharedSandbox tier** — active. Contract `014`
  owns semantics. Batch 2.0 froze multiplexing at
  `docs/architecture/shared-sandbox-multiplexing.md`. v1 grouping is plugin
  type identity.

## Why this generation matters now

`g11.001` closed scan → placement → bridge backend → offline render-plane.
`g11.002` implements the shared-boundary tier the runtime already names.

## Generation runway

1. **`g11.001`** — production host-assembly wiring (complete)
2. **`g11.002`** — SharedSandbox tier (active; Batch 2.1 ready)
3. **Backlog pulls** — graph successor, device depth, analysis breadth only when
   Loophole or another consumer names the dependency

Do not reopen stretch, Automatic, or RealtimePreview adoption work from this
generation.

## Milestone Map

- `g11.001` `complete`
  - production host-assembly wiring; Batches 1.1–1.4 closed
- `g11.002` `active`
  - SharedSandbox tier. Batch 2.0 complete. Batch 2.1 ready.

## Working Rule

- treat Contract `072` as the hosting baseline authority
- treat Contract `014` as the SharedSandbox semantics authority
- treat `docs/architecture/shared-sandbox-multiplexing.md` as the v1
  implementation map
- keep `signal-host-local` a thin wrapper over runtime-owned meaning (Contract
  `009`)
- do not infer Loophole UI, Chorus mixer, or downstream workflow scope

## Next Task

Execute `docs/roadmaps/g11/batch-cards/005-g11-002-broker-multiplexing.md`.
Authority: Contract `014` and
`docs/architecture/shared-sandbox-multiplexing.md`.
