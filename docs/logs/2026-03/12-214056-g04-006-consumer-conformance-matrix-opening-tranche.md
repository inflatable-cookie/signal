# 12-214056 g04.006 Consumer Conformance Matrix Opening Tranche

Status: complete
Owner: core-product
Related roadmap: `docs/roadmaps/g04/006-consumer-conformance-export-stability-and-release-packaging.md`

## Summary

Completed `g04.006` Batch 6.1 by defining the first runnable consumer
conformance matrix for the stabilized runtime/export/plugin boundary and
exposing that matrix through repo-owned tooling.

## Work Completed

- added `signal-supervisor-tools --describe-conformance-matrix` in
  `crates/signal-supervisor-tools/src/main.rs`
- defined the initial runnable matrix around:
  - the `signal-runtime` downstream public-boundary proof
  - the `signal-supervisor-tools` export-consumer proof
  - the host-free `signal-runtime` supervisor report example
  - the conformance-matrix introspection surface itself
- added `effigy acceptance:conformance` so the same matrix is
  runnable through a repo-owned task without private implementation detail
- updated roadmap/reference docs to point the queue at Batch 6.2 packaging
  work

## Validation

- `cargo test -p signal-supervisor-tools parse_args_supports_describe_conformance_matrix_mode`
- `cargo test -p signal-supervisor-tools conformance_matrix_json_reports_runnable_consumer_boundary`
- `cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json`
- `effigy acceptance:conformance`

## Residual Risk

The consumer conformance matrix is now runnable, but release packaging,
versioning policy, and artifact expectations are still implicit until the
Batch 6.2 packaging baseline is defined.

## Next Task

Continue `g04.006` with Batch 6.2 by defining the first release-packaging and
versioning baseline on top of the runnable consumer conformance matrix.
