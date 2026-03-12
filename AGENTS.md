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

## References

- `../chorus/specs/guidelines/agents-operating-guardrails.md`
- `../chorus/specs/`
