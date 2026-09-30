# Signal

Signal is the shared realtime audio library and runtime for Loophole, Finch,
and other consumers. It owns reusable DSP, analysis, graph/runtime semantics,
plugin discovery and hosting, and audio output. Project editing and UI belong
to downstream products.

Read [docs/README.md](docs/README.md), then the owning file in
[docs/knowledge/](docs/knowledge/README.md). Intent is in
`plan.md` (Git history). Queue holds tasks, briefs, status, review,
closeout, and outcomes. Triage notes are leads, not authority.

## Boundaries

- On `signal-render-plane`, `signal-hardware` callbacks, and DSP kernels
  they call: no allocation, blocking, locks, or unbounded work.
  `signal-runtime` allocates by design.
- Treat plugin code as untrusted. Keep plugin and hardware glue at the edge;
  reusable processing stays in owning crates.
- Keep IPC and message contracts aligned with Chorus specs. Consult the
  sibling guardrails at
  `../chorus/specs/guidelines/agents-operating-guardrails.md`. When Signal's
  contracts do not settle a question and that checkout is absent, stop rather
  than infer foreign error meaning.
- Keep crate and module responsibilities narrow. Prefer real end-to-end
  behavior and typed degraded outcomes over scaffolds or hidden fallback.
- Avoid compatibility shims unless a governing contract or the operator
  requires one. Keep Signal work separate from Loophole UI and Pulse state.
- Do not run release mutations or edit `.github/workflows/` without an
  explicit operator request.

## Work

- Use the current checkout for normal work. Queue supplies worker workspaces
  and briefs; do not create repository task cards or handoff files.
- Work in meaningful batches. A docs review does not authorize product-code
  changes. Rust audit work records scope and findings before repair.
- Keep knowledge current in the same change as behavior. One owner per fact.
  Record unresolved questions in `docs/knowledge/questions.md`.
- File papercuts in Queue with `papercut.add` (see the `northstar`
  skill). The repository holds no papercut file or triage folder.
- Write plainly; see `docs/policy/internal-writing-style.md`.

## Effigy guidance

Use the maintained installed `effigy` Agent Skill for shared guidance.
Install it with `npx skills add inflatable-cookie/effigy -g` if absent.
Supported roots are `~/.agents/skills/effigy`, `~/.codex/skills/effigy`,
`~/.claude/skills/effigy`, and `~/.cursor/skills/effigy`; symlink aliases must
resolve to one canonical directory. Stop on conflicting installs. Re-read its
`SKILL.md` and confirm discovery in a fresh agent context after a refresh.
Keep Signal selectors and guardrails here and in owning knowledge.

The executable is separate: check `command -v effigy` and
`effigy admission status --json` for schema `effigy.admission.status.v1`.
If missing or unsupported, install an admission-capable Effigy build using
Effigy's installation guidance before validation. Plain `effigy init` does
not install the skill; do not restore a frozen project copy.

## Validation

Use Effigy for supported work. Use `effigy graph` for code understanding,
`effigy tasks` for selector inventory, `effigy doctor` for routing ambiguity,
and `effigy test --plan` when test shape matters. Use `--json` when another
tool consumes output. Do not add a current-directory repo override.

Run targeted checks once per task. `effigy validate` builds, formats, and
compile-checks; select only the checks needed for the changed code.
After docs changes run `effigy qa:docs`, which includes `qa:northstar`.
`qa:docs:agent-defaults` remains a separate check. Queue runs the full
`effigy qa` board on `main` at milestones.

Rust source, manifests, build files, tests, and related docs follow the
repository-owned quality profile and deviations in `docs/knowledge/contracts/`.
Re-enter that profile at task start and batch closeout. Preserve unrelated work.
An explicit quality audit uses the audit route, not everyday authoring.
