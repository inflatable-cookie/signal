# Papercuts

Small, actionable friction found during agent work. Agents append entries when
they hit a solvable hurdle; they do not stop the current task to fix one.

## Open

<!-- Keep entries short. Append newest entries at the top. Do not include secrets. -->

### [ ] Cross-repo worker handoff paths need resolution — 2026-08-21
- Friction: Soundcheck exposed the ready card, but the implementation handoff
  lived in the sibling Signal repo; a relative lookup first reported it absent.
- Impact: worker startup spent a pass locating the owning repository and its
  dedicated worktree.
- Possible fix: resolve `handoff_path` across linked repositories before
  launching a worker.
- Surface: Soundcheck/Signal handoff dispatch

### [ ] SharedSandbox sequential prepare must stop before load — 2026-08-17
- Friction: broker rejects `load-plugin-instance` while `already_processing`.
  Host factory stops the boundary, adds the member, then starts again.
- Impact: sequential SharedSandbox prepares work; live add-while-processing
  stays out of v1 (design non-goal).
- Possible fix: keep stop/start as host orchestration, or later add a
  broker-side pause that is still not audio-thread member add.
- Surface: `signal-host-local` factory, `signal-plugin-sandbox` broker

### [ ] Signal had no `.agents.local.env` at first orchestrator dispatch — 2026-08-17
- Friction: worker fallback worktrees cannot be created without
  `AGENTS_WORKTREE_CONTAINER_DIR`.
- Impact: dispatch would stop at the local-path contract.
- Possible fix: keep `.agents.local.env` gitignored and seeded on this machine
  (`/Users/tom/Dev/worktrees`, matching sibling repos).
- Surface: `.agents.local.env`, orchestrator worker fallback

## Closed

### [x] Stale plugin-hosting docs misled planning — 2026-08-17
- Friction: post-demolition backlog and architecture front doors still said
  plugin hosting was missing after `g09`/`g11`/`g12` shipped real CLAP/VST3/AU/LV2
  hosting.
- Impact: refresh/atlas work reopened a finished lane and wasted operator time.
- Fix: verified Contract `072`, backlog, architecture, and strategic runway on
  this SHA already state CLAP/VST3/AU/LV2 hosting is shipped; runway now names
  closed `g11.001`/`g11.002` integration rather than remaining hosting gaps.
- Surface: docs/architecture, docs/roadmaps/backlog, docs/contracts/072

### [x] Northstar refresh found stale Next Task pointers — 2026-08-17
- Friction: roadmap front doors still pointed at finished work (`g10` stretch
  audit, then later `g11.002` PR review) after those lanes closed.
- Impact: a bare `continue` or agent re-entry could reopen finished lanes.
- Fix: aligned `docs/roadmaps/README.md`, `g11` front doors, generation index,
  and strategic runway Next Task to operator backlog selection + Soundcheck
  card 135 wait; left closed milestone/card closeouts historical.
- Surface: docs/roadmaps front doors

### [x] VST3 module binary resolution is cfg-gated off macOS — 2026-08-22
- Friction: `resolve_module_binary_path` only compiled on non-macOS, so
  Windows `Contents/{x86_64,arm64}-win` tests could not run here.
- Impact: platform-parameterized Windows layout tests needed
  `cfg(any(test, not(target_os = "macos")))`.
- Fix: compile path resolution on every host; keep dlopen/hosting cfg-gated.
- Surface: `signal-plugin-vst3` introspection/paths.rs

### [x] Host-edge parity tests hardcode platform-coverage length — 2026-08-17
- Friction: adding LV2 to `LocalRuntimeHost` platform coverage broke
  `parity_coverage.len() == 3` in a public host-edge test.
- Impact: owning a fourth format looks like a product regression.
- Fix: assert owned formats by record instead of a magic length.
- Surface: `crates/signal-host-local/tests/public_host_edge_cross_adapter_parity.rs`
