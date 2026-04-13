# AGENTS

Scope: `signal/`.

## Hard Rules

- Preserve real-time safety constraints in audio-thread paths.
- Keep IPC/message contracts aligned with Chorus specs.
- Keep module responsibilities narrow and explicit.
- Avoid compatibility shims unless explicitly requested.

## Effigy-First Execution

- Start with `effigy tasks` to inspect Signal's local task surface.
- Run `effigy doctor` when environment or task resolution is uncertain.
- Prefer `effigy health` as the default repo-owned baseline.
- Prefer local Effigy tasks such as `effigy build`, `effigy dev`, `effigy test --plan`, `effigy validate`, and `effigy qa:docs`.
- Fall back to raw CMake or CTest commands only when the needed operation is not represented in `effigy.toml`.

## Validate

- `effigy health`
- `effigy validate`
- `effigy qa:docs` when docs or planning surfaces change
- `effigy test --plan` before test-focused work

## Strict-Lane Continuation

When Signal is operating inside a strict Northstar lane, a bare `continue`
should be enough.

Treat it as:

- resume from the previous closeout's `Next Task`
- re-anchor on the current ready batch card or explicit stop/reassessment step
- stay inside that bounded lane unless the file state itself requires a stop

If the previous `Next Task` does not point at a real ready card or explicit
reassessment step, do not infer the next move from memory. Re-enter planning
from the active docs surfaces first.

## References

- `../chorus/specs/guidelines/agents-operating-guardrails.md`
- `../chorus/specs/`

## Internal Writing Style

Use the repo-local style reference for internal work and normal replies:

- `docs/policy/internal-writing-style.md`
