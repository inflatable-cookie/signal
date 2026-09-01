# Signal

Scope: `signal/`.

Signal is the shared realtime audio library and runtime for the Loophole
ecosystem. It owns reusable DSP, analysis, graph/runtime semantics, plugin
discovery and hosting, and audio output. It does not own project editing, UI,
or downstream product workflow.

## Non-negotiable boundaries

- Preserve realtime safety on audio-thread paths: no allocation, blocking,
  locks, or unbounded work. Those paths are the `signal-render-plane`
  executor, the `signal-hardware` capture/output callbacks, and the DSP
  kernels they call — not `signal-runtime`, which allocates by design.
- Treat plugin code as untrusted. Keep plugin and hardware glue at the edge;
  keep reusable signal processing inside the owning crates.
- Keep IPC and message contracts aligned with Chorus specs. Do not change an
  unresolved contract or foreign error meaning silently.
- Keep crate and module responsibilities narrow and explicit. Prefer real
  end-to-end behavior and typed degraded outcomes over scaffolds or hidden
  fallback.
- Avoid compatibility shims unless the governing contract or the operator
  explicitly requires one.
- Keep Signal-owned work separate from Loophole UI, Pulse state ownership,
  and other downstream workflow.
- Do not run release mutations or change `.github/workflows/` without an
  explicit operator request.

## Work and authority

- Use the canonical docs and ready-card surfaces. Do not invent a parallel
  planning authority.
- Start with `docs/README.md`, then follow the relevant architecture,
  contract, roadmap, and evidence surfaces. If no active card or planning
  authority settles the next direction, stop and ask.
- A strict lane is valid only while its current card and governing refs match
  live state. In that lane, a bare `continue` follows the previous closeout's
  `Next Task`; without a ready card, re-enter planning instead of guessing.
- Normal-mode agents use the current checkout. Worker mode exists only after
  an explicit orchestrator-dispatched handoff under `docs/handoffs/`; never
  infer it from a branch, path, or harness. Operator-facing dispatch is that
  handoff's absolute path in the owning repo (Northstar
  `1840c9f6d4f7127240622a09e462b06adc094971`), not a Soundcheck-relative
  lookup.
- Work in meaningful batches. Refresh, docs review, and AGENTS review do not
  authorize production-code changes; Rust audit work must record scope and
  findings before repair.

## Route work by job

The Effigy Agent Contract at the end of this file covers routing itself —
`graph`, `tasks`, `doctor`, `test --plan`, and `--json`. Signal adds:

- `effigy validate` is the normal build/format/compile-validation surface and
  `effigy qa` is full local QA;
- run `effigy qa:docs` and `effigy qa:northstar` after docs or planning
  changes;
- fall back to raw Cargo only when the operation is not in `effigy.toml`, and
  never add a package script that merely re-exports Effigy.

Signal has no target-local `check:agent-instructions` task. AGENTS review
uses the installed Northstar consumer-safe audit:

```sh
effigy --repo <installed-northstar> northstar/check:agent-instructions <this-repo>
```

`qa:docs:agent-defaults` stays a separate check; it forbids a current-directory
repo override on the instruction surface.

## Canonical surfaces

- `docs/README.md` — project and docs front door;
- `docs/architecture/` — system shape and ownership;
- `docs/contracts/` — durable boundaries and policy;
- `docs/roadmaps/README.md` and the active generation front door — queue and
  next-task authority;
- `docs/logs/README.md` — evidence history;
- `docs/triage/` — unresolved leads to promote, merge, keep open with an owner,
  or remove; never execution authority;
- `docs/policy/internal-writing-style.md` — glue-light internal writing;
- `.agents/skills/effigy/SKILL.md` — repository-local Effigy routing and
  operating details.

When changing IPC, consult the sibling Chorus guardrails at
`../chorus/specs/guidelines/agents-operating-guardrails.md`. That checkout is
often absent here; if it is, and Signal's own contracts do not settle the
question, record the limitation and stop rather than inferring the contract.
Nested `AGENTS.md` files, contracts, and skills add path-specific rules.

## Completion

A change is complete only when the claimed behavior is real, relevant
architecture/contracts/roadmaps/logs reflect the truth, validation actually
ran, and remaining limits are explicit. Leave one useful `Next Task` in the
highest-authority active surface. During execution, append a small recurring,
solvable hurdle to `PAPERCUTS.md`; do not turn that observation into unplanned
work.

<!-- BEGIN EFFIGY AGENT CONTRACT -->
## Effigy Agent Contract

Use Effigy as the default command surface for supported project work.
Route by job, not by startup ritual:
- use `effigy graph` for code understanding
- use `effigy tasks` for selector inventory
- use `effigy doctor` for routing ambiguity or repo health
- use `effigy test --plan` when test execution shape matters

Use `effigy graph` when the job is code understanding: ownership, behavior,
implementation, or changed-file impact. Do not insert graph into unrelated
deployment, state, docs, release, or direct task-execution work.

Prefer `effigy <task>`, `effigy test`, and the matching built-in surface over
raw package-manager or shell commands when Effigy covers the path. Use
`effigy --json <command>` whenever another agent or tool will consume output.

Do not add a current-directory repo override while already inside the target
repo. Do not edit `.github/workflows/` or run release mutations unless the
operator explicitly asks.
<!-- END EFFIGY AGENT CONTRACT -->

<!-- northstar:rust-quality:start -->
## Northstar Rust Quality

Scope: Rust source, Cargo manifests, build files, tests, and directly related
documentation under this directory.

Use Northstar's strict everyday-authoring route for ordinary Rust work. Resolve
the repository-owned profile and deviations under `docs/contracts/`; never
assume a universal MSRV. Re-enter at task start and coherent batch closeout.
Preserve unrelated work. A quality audit, no-slop pass, or audit-and-fix request
is explicit audit intent; never route it through everyday authoring.
<!-- northstar:rust-quality:end -->
