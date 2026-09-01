# Papercuts

Small, actionable friction found during agent work. Agents append entries when
they hit a solvable hurdle; they do not stop the current task to fix one.

## Open

<!-- Keep entries short. Append newest entries at the top. Do not include secrets. -->

### [ ] Rust audit `collect` fabricates `unrun` records on a partial second call — 2026-09-01
- Friction: evidence records are immutable, so a second `collect` for one unit
  must omit already-recorded requests. The tool then treats those classes as
  unrepresented in that call and writes `unrun-<class>-<unit>` for **every**
  unit in the audit — 42 false "evidence unrun" limitations that contradict the
  passing records already on disk.
- Impact: the finalized report misrepresents validation state. There is no way
  to remove the fabricated records, so the whole audit has to be restarted.
- Fix: collect each unit exactly once, with every applicable class represented
  in that single plan. If a class genuinely cannot run, send it as an explicit
  `unrun`/`unavailable` request in the same call rather than omitting it.
- Surface: installed Northstar `rust-quality` `collect`; `g11.003` audit lane

## Closed

### [x] SharedSandbox sequential prepare must stop before load — 2026-08-17
- Friction: broker rejects `load-plugin-instance` while `already_processing`.
  Host factory stops the boundary, adds the member, then starts again.
- Impact: sequential SharedSandbox prepares work; live add-while-processing
  stays out of v1 (design non-goal).
- Fix: host path already stop → load/activate → start
  (`prepare_shared_sandbox_processor`). Added broker regression
  `load_while_processing_rejects_until_boundary_stop` pinning the refusal
  token and post-stop sequential add. Evidence:
  `docs/logs/2026-08/31-papercuts-wave26-sharedsandbox-stop-before-load.md`.
- Surface: `signal-host-local` factory, `signal-plugin-sandbox` broker

### [x] Cross-repo worker handoff paths need resolution — 2026-08-30
- Friction: Soundcheck exposed the ready card, but the implementation handoff
  lived in the sibling Signal repo; a relative lookup first reported it absent.
- Impact: worker startup spent a pass locating the owning repository and its
  dedicated worktree.
- Fix: proved against Northstar
  `1840c9f6d4f7127240622a09e462b06adc094971` (PR 8); operator-facing dispatch
  is the owning repo's absolute handoff path. `AGENTS.md` states that; no
  cross-repo path resolver and no Soundcheck file copies.
- Surface: Soundcheck/Signal handoff dispatch; `AGENTS.md`

### [x] Signal had no `.agents.local.env` at first orchestrator dispatch — 2026-08-30
- Friction: worker fallback worktrees cannot be created without
  `AGENTS_WORKTREE_CONTAINER_DIR`.
- Impact: dispatch would stop at the local-path contract.
- Fix: `.agents.local.env` exists locally, is gitignored
  (`.gitignore:72:.agents.local.env`), and sets
  `AGENTS_WORKTREE_CONTAINER_DIR=/Users/tom/Dev/worktrees`. File is not
  staged or committed.
- Surface: `.agents.local.env`, orchestrator worker fallback

### [x] Rust quality setup scope is repository-relative — 2026-08-30
- Friction: the managed setup rejected an absolute scope directory even when the target root was absolute.
- Impact: the first setup invocation failed before profile discovery.
- Fix: proved against sibling Northstar `77dcda9fa20e9d63977eb3488b0738ea0391f0bb` (PR 9); `northstar/rust-quality:setup apply <abs-target> <abs-target>` exits 0 and stores scope as `.`. Aligned the existing AGENTS activation block to the installed template wrapping so apply could stay idempotent.
- Surface: Northstar `rust-quality:setup`; `AGENTS.md` activation markers

### [x] Consumer AGENTS audit is not exposed through the local catalog — 2026-08-30
- Friction: the target-local `effigy check:agent-instructions` selector is absent, so the installed Northstar catalog is required for the read-only audit.
- Impact: agent-instruction review needs a fallback command and an extra routing check.
- Fix: documented the installed-Northstar consumer-safe command on `AGENTS.md`; no local Rhai copy. `qa:docs:agent-defaults` stays separate.
- Surface: `AGENTS.md`, AGENTS review routing

### [x] Stale plugin-hosting docs misled planning — 2026-08-17
- Friction: post-demolition backlog and architecture front doors still said
  plugin hosting was missing after `g09` shipped real CLAP/VST3/AU/LV2 hosting
  and `g11` closed host-assembly / SharedSandbox integration.
- Impact: refresh/atlas work reopened a finished lane and wasted operator time.
- Fix: verified Contract `072`, backlog, architecture, and strategic runway on
  this SHA already state CLAP/VST3/AU/LV2 hosting is shipped; runway now names
  closed `g11.001`/`g11.002` integration rather than remaining hosting gaps.
- Surface: docs/architecture, docs/roadmaps/backlog, docs/contracts/072

### [x] Northstar refresh found stale Next Task pointers — 2026-08-17
- Friction: live docs/architecture/contracts/roadmap front doors still pointed
  at finished work (`g10` stretch audit, `g11.002` / SharedSandbox PR review,
  then Soundcheck card 135 / Linux-Windows CLAP waits after `086`/`087`
  shipped).
- Impact: a bare `continue` or agent re-entry could reopen finished lanes.
- Fix: aligned live Next Task surfaces — including Contract `001` — to
  operator backlog selection; recorded Linux CLAP (`086`) and Windows CLAP
  (`087`) as shipped; left closed milestone/card/log closeouts historical.
- Surface: docs README, architecture front doors, contracts README + `001`,
  roadmaps front doors

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
