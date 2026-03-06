# AGENTS

Scope: `signal/`.

## Hard Rules

- Preserve real-time safety constraints in audio-thread paths.
- Keep IPC/message contracts aligned with Chorus specs.
- Keep module responsibilities narrow and explicit.
- Avoid compatibility shims unless explicitly requested.

## Effigy-First Execution

- Start with `effigy tasks --repo .` to inspect Signal's local task surface.
- Prefer `effigy health --repo .` as the default repo-owned baseline.
- Prefer local Effigy tasks such as `effigy build --repo .`, `effigy dev --repo .`, `effigy test --repo .`, and `effigy validate --repo .`.
- Fall back to raw CMake or CTest commands only when the needed operation is not represented in `effigy.toml`.

## Validate

- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .` when behavior changes

## References

- `../chorus/specs/guidelines/agents-operating-guardrails.md`
- `../chorus/specs/`
