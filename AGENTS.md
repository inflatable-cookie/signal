# AGENTS

Scope: `signal/`.

## Always-loaded boundaries

- Preserve real-time safety constraints in audio-thread paths.
- Keep IPC/message contracts aligned with Chorus specs.
- Keep module responsibilities narrow and explicit.
- Avoid compatibility shims unless explicitly requested.
- Use Signal's canonical docs and ready-card surfaces; do not invent parallel
  planning authority.
- Normal-mode agents use the current checkout. Worker mode activates only from
  an explicit orchestrator-dispatched handoff under `docs/handoffs/`.
- If planning authority does not settle the next direction, stop and ask. Stay
  inside the current bounded lane.
- Do not run release mutations or change CI/workflow files without an explicit
  request.

## Common commands

Route by job, not by startup ritual:

```sh
effigy tasks
effigy doctor       # only when routing or environment state is uncertain
effigy health       # cheap repo-owned baseline
effigy validate
effigy qa:docs      # when docs or planning surfaces change
```

Prefer `effigy <task>` for supported work. Use `effigy graph` for code
understanding. Use `effigy test --plan` when test execution shape matters.
Fall back to raw CMake or CTest only when the needed operation is not in
`effigy.toml`.

This repo's local `.agents/skills/effigy` copy is authoritative. When an agent
supports both project-local and global skills, prefer the project-local copy.

## Docs authority

- `docs/README.md`
- `docs/roadmaps/README.md`
- `docs/roadmaps/strategic-runway.md`
- `docs/logs/README.md`
- `docs/contracts/001-working-rules.md`
- `docs/contracts/contract-index.md`

During execution, record a small recurring solvable hurdle in `PAPERCUTS.md`;
do not make that observation unplanned work.

## Strict-lane continuation

When Signal is operating inside a strict Northstar lane, a bare `continue`
should be enough:

- resume from the previous closeout's `Next Task`
- re-anchor on the current ready batch card or explicit stop/reassessment step
- stay inside that bounded lane unless the file state itself requires a stop

If the previous `Next Task` does not point at a real ready card or explicit
reassessment step, do not infer the next move from memory. Re-enter planning
from the active docs surfaces first.

## Read on demand

- Chorus guardrails: `../chorus/specs/guidelines/agents-operating-guardrails.md`
- Internal writing style: `docs/policy/internal-writing-style.md`
- Effigy adoption: `docs/guides/047-agent-and-cross-repo-adoption.md`
- Graph workflows: `docs/guides/076-code-graph-and-agent-workflows.md`
- JSON contracts: `docs/guides/017-json-output-contracts.md`
- Nested `AGENTS.md` files, contracts, guides, and skills for path-specific work

<!-- BEGIN EFFIGY AGENT CONTRACT -->
## Effigy Agent Contract

Use Effigy as the default command surface for supported project work.

Route by job, not by startup ritual:
- use `effigy graph` for code understanding
- use `effigy tasks` for selector inventory
- use `effigy doctor` for routing ambiguity or repo health
- use `effigy test --plan` when test execution shape matters

Use `effigy graph` when the job is code understanding: ownership, flow,
implementation, or changed-file impact. Do not insert graph into unrelated
deployment, state, docs, release, or direct task-execution work.

Prefer `effigy <task>`, `effigy test`, and the matching built-in surface over
raw package-manager or shell commands when Effigy covers the path. Use
`effigy --json <command>` whenever another agent or tool will consume output.

Do not add a current-directory repo override while already inside the target repo. Do not edit
`.github/workflows/` or run release mutations unless the user explicitly asks.
<!-- END EFFIGY AGENT CONTRACT -->
