# AGENTS

Scope: `signal/`.

## Hard Rules

- Preserve real-time safety constraints in audio-thread paths.
- Keep IPC/message contracts aligned with Chorus specs.
- Keep module responsibilities narrow and explicit.
- Avoid compatibility shims unless explicitly requested.

## Validate

- Run existing CMake/CTest workflow when C++ changes.

## References

- `../chorus/meta/260-agents-operating-guardrails.md`
- `../chorus/specs/`
