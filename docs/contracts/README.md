# Contracts

Use this folder for explicit non-code contracts that constrain behavior.

Examples:
- protocol contracts
- API behavior contracts
- policy contracts

Current baseline:
- `001-shared-dsp-and-host-boundary.md`
- `002-supervisor-export-schema-and-report-boundary.md`

## Rule

Contracts should be stable reference artifacts and link to relevant roadmap/log evidence.

## Next task

Use `001-shared-dsp-and-host-boundary.md` as the initial rule set, then add
new contracts only where a boundary needs explicit guarantees beyond what the
architecture doc already states.
