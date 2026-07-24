# g10.034 Continuous Cyclic Compile Repair

Date: 2026-07-25
Batch: 34.3
Status: active; pre-checkpoint evidence repair

## Finding

Effigy `v0.8.17` accepts trailing arguments on a generic task invocation but
does not forward them to the task command. The frozen command therefore ran
the configured plain `cargo build --workspace`; it did not create the required
release-only evidence binary or use the named build root.

## Repair

The candidate compile authority now runs:

1. `effigy build`
2. the narrow raw Cargo release build with explicit package, binary, and
   absolute `CARGO_TARGET_DIR`

Signal has no local Effigy selector for that second shape. This is the
repo-authorized fallback for an unrepresented operation.

No renderer, threshold, source, comparator, receipt, gate, or promotion rule
changed. No acoustic row ran. Contract `085` Rule 11 applies.

## Next

Rebase the isolated candidate onto this docs-only repair. Commit one clean
candidate tree, run two complete conformance rounds, and create the acoustic
ref only after both pass unchanged.
