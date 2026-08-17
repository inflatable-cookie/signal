# g11 Milestones

Status: `g11.001` complete; `g11.002` complete
Updated: 2026-08-17

## At a glance

- `g11` is the post-stretch integration generation: turn shipped plugin-hosting
  substrate into a trustworthy Pulse-facing production path.
- Plugin hosting baseline work is **not** in scope — CLAP, VST3, AU, and LV2
  hosting already ship through adapter crates, `signal-plugin-sandbox`, and
  `signal-plugin-bridge`. See Contract `072`.
- First milestone: **`g11.001` production host-assembly wiring** — complete.
- Second milestone: **`g11.002` SharedSandbox tier** — complete. Contract `014`
  owns semantics. v1 grouping is plugin type identity. Map:
  `docs/architecture/shared-sandbox-multiplexing.md`.

## Why this generation matters now

`g11.001` closed scan → placement → bridge backend → offline render-plane.
`g11.002` closed the shared-boundary tier the runtime already names.

## Generation runway

1. **`g11.001`** — production host-assembly wiring (complete)
2. **`g11.002`** — SharedSandbox tier (complete)
3. **Backlog pulls** — graph successor, device depth, analysis breadth only when
   Loophole or another consumer names the dependency

Do not reopen stretch, Automatic, or RealtimePreview adoption work from this
generation.

## Milestone Map

- `g11.001` `complete`
  - production host-assembly wiring; Batches 1.1–1.4 closed
- `g11.002` `complete`
  - SharedSandbox tier. Batches 2.0–2.3 closed.

## Working Rule

- treat Contract `072` as the hosting baseline authority
- treat Contract `014` as the SharedSandbox semantics authority
- treat `docs/architecture/shared-sandbox-multiplexing.md` as the v1
  implementation map
- keep `signal-host-local` a thin wrapper over runtime-owned meaning (Contract
  `009`)
- do not infer Loophole UI, Chorus mixer, or downstream workflow scope

## Next Task

Stop for operator review of the `g11.002` PR. Do not start a follow-on
generation. The next `g11` item is a product-pulled backlog pull, not an
implied `g12`.
