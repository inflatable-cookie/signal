# 2026-04-10 - g09.014 Workspace Validate Surface Repair

## Summary

Closed `037-g09-014-workspace-validate-surface-repair.md` by repairing the
broken workspace validate wall in the host crates. The stale split test-module
tree and related host-test import drift in `signal-host-local` and
`signal-host-server` no longer prevent the workspace-wide compile sweep from
finishing, so the reopened `g09` release gate can treat `effigy validate` as a
real required signal again.

## Implementation

- repaired the stale split test-module declarations in the local and server
  host test trees so the nested recovery, report, soak, and public-boundary
  modules resolve correctly again
- repaired the related host-test import, helper-path, and report-type drift
  surfaced by the same compile wall
- revalidated the underlying workspace no-run sweep directly instead of only
  trusting the aggregate Effigy wrapper
- updated the `g09.014` release-gate docs to promote `effigy validate` from
  deferred to required evidence
- promoted the next bounded readiness batch as the plugin, broker, and IPC
  family verdict card

## Validation

- `cargo test -p signal-host-local --lib --no-run`
- `cargo test -p signal-host-server --lib --no-run`
- `effigy validate`
- `cargo test --workspace --no-run`

## Notes

- the workspace validate wall is green again, but the sweep still emits
  non-blocking warning clusters in `signal-plugin-clap`, `signal-runtime`,
  `signal-host-local`, and `signal-host-server`
- those warnings are now normal cleanup debt, not a broken gate surface

## Next Task

Continue the reopened strict `g09` lane from
`docs/specs/batch-cards/038-g09-014-plugin-broker-readiness-verdict.md`.
