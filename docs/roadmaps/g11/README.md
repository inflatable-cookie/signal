# g11 Tasks

Updated: 2026-09-09

## At a glance

- `g11` is the post-stretch integration generation: turn shipped plugin-hosting
  substrate into a trustworthy Pulse-facing production path.
- Plugin hosting baseline work is **not** in scope — CLAP, VST3, AU, and LV2
  hosting already ship through adapter crates, `signal-plugin-sandbox`, and
  `signal-plugin-bridge`. See Contract `072`.
- First task: **`g11.001` production host-assembly wiring** — complete.
- Second task: **`g11.002` SharedSandbox tier** — complete. Contract `014`
  owns semantics. v1 grouping is plugin type identity. Map:
  `docs/architecture/shared-sandbox-multiplexing.md`.
- Maintenance task: **`g11.003` Northstar instruction and Rust quality
  audit** — complete and merged through PR `#18`; no product behavior
  or new generation.

## Why this generation matters now

`g11.001` closed scan → placement → bridge backend → offline render-plane.
`g11.002` closed the shared-boundary tier the runtime already names.

## Generation runway

1. **`g11.001`** — production host-assembly wiring (complete)
2. **`g11.002`** — SharedSandbox tier (complete)
3. **`g11.003`** — Northstar instruction and Rust quality audit (complete and
   merged through PR `#18`)
4. **Backlog pulls** — graph successor, device depth, analysis breadth only when
   Loophole or another consumer names the dependency

Do not reopen stretch, Automatic, or RealtimePreview adoption work from this
generation.

## Task Map

- `g11.001` `complete`
  - production host-assembly wiring
  (`001-production-host-assembly-wiring.md`)
- `g11.002` `complete`
  - SharedSandbox tier
  (`002-shared-sandbox-tier.md`)
- `g11.003` `complete`
  - repository-scope AGENTS and Rust audit, merged through PR `#18`.
    Audit `signal-g11-003-repository-audit`, 14 units over 28 crates, 89
    recorder-authorized repairs, 8 unsafe findings left report-only.
  (`003-northstar-instruction-and-rust-quality-audit.md`)

Earlier milestone wrappers and nested batch cards were absorbed into these
three tasks by the flattened-task migration; the generation README is the
single roadmap and frontier, and each `g11.NNN` file above is the sole
executable planning unit for its task.

## Working Rule

- treat Contract `072` as the hosting baseline authority
- treat Contract `014` as the SharedSandbox semantics authority
- treat `docs/architecture/shared-sandbox-multiplexing.md` as the v1
  implementation map
- keep `signal-host-local` a thin wrapper over runtime-owned meaning (Contract
  `009`)
- do not infer Loophole UI, Chorus mixer, or downstream workflow scope

## Queue lifecycle adoption

- [g11.004 Effigy-hosted lifecycle hook](004-adopt-effigy-hosted-lifecycle-hook.md)
  is an operator-approved, configuration-only maintenance lane. It follows its
  declared Queue dependencies and may run without changing product priority.
  Existing next-task text continues to describe product sequencing; this entry
  authorizes no sibling product work.

## Next Task

Return to operator planning or triage review. `g11` has no ready task; do
not open `g12` or infer a product pull from triage without operator selection.
<!-- northstar:lifecycle:begin schema=northstar.lifecycle.projection.v2 digest=sha256:285994a4b995788f61a095359a0b150bd645f28e8c3e76709eb801657f91a878 -->
| Generation | Disposition | Runway state |
| --- | --- | --- |
| g11 | open | planning_required |
| Task | Status | Stage | Revision | Record digest |
| --- | --- | --- | --- | --- |
| g11.004 | complete | none | 8 | sha256:900e068a968d40ef2754db498912bc2e7d649b1607721a059615b905acdc4f09 |
| g11.005 | complete | none | 8 | sha256:e3020be4ff11a71313025c50cbc857a2f1dfe6c8d5043eb4ac3c70e8c47cdf2f |
<!-- northstar:lifecycle:end -->
