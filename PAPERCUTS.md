# Papercuts

Small, actionable friction found during agent work. Agents append entries when
they hit a solvable hurdle; they do not stop the current task to fix one.

## Open

<!-- Keep entries short. Append newest entries at the top. Do not include secrets. -->

### [ ] Signal had no `.agents.local.env` at first orchestrator dispatch — 2026-08-17
- Friction: worker fallback worktrees cannot be created without
  `AGENTS_WORKTREE_CONTAINER_DIR`.
- Impact: dispatch would stop at the local-path contract.
- Possible fix: keep `.agents.local.env` gitignored and seeded on this machine
  (`/Users/tom/Dev/worktrees`, matching sibling repos).
- Surface: `.agents.local.env`, orchestrator worker fallback


### [ ] Stale plugin-hosting docs misled planning — 2026-08-17
- Friction: post-demolition backlog and architecture front doors still said
  plugin hosting was missing after `g09`/`g11`/`g12` shipped real CLAP/VST3/AU/LV2
  hosting.
- Impact: refresh/atlas work reopened a finished lane and wasted operator time.
- Possible fix: keep Contract `072`, backlog, architecture, and strategic runway
  aligned whenever hosting depth lands.
- Surface: docs/architecture, docs/roadmaps/backlog, docs/contracts/072

### [ ] Northstar refresh found stale Next Task pointers — 2026-08-17
- Friction: `docs/roadmaps/README.md` and `docs/roadmaps/g10/README.md` still
  pointed at pre-closeout stretch-audit work after `g10.036`–`g10.042` closed.
- Impact: a bare `continue` or agent re-entry could reopen finished lanes.
- Possible fix: keep generation front doors and roadmap README aligned whenever
  a milestone suite closes.
- Surface: docs/roadmaps front doors

## Closed

### [x] Host-edge parity tests hardcode platform-coverage length — 2026-08-17
- Friction: adding LV2 to `LocalRuntimeHost` platform coverage broke
  `parity_coverage.len() == 3` in a public host-edge test.
- Impact: owning a fourth format looks like a product regression.
- Fix: assert owned formats by record instead of a magic length.
- Surface: `crates/signal-host-local/tests/public_host_edge_cross_adapter_parity.rs`
