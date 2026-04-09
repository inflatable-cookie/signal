# g09.001 Planning Freeze And g09.002 Discovery Matrix Tranche

Status: recorded
Owner: core-product
Date: 2026-04-08
Related roadmaps: `docs/roadmaps/g09/001-audit-planning-surfaces-and-remediation-contract-freeze.md`, `docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
Related contracts: `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/contracts/073-native-backend-device-truth-and-coreaudio-implementation-contract.md`, `docs/contracts/074-shared-host-runtime-execution-and-recovery-unification-contract.md`, `docs/contracts/075-runtime-public-interface-decomposition-and-internal-assembly-boundary-contract.md`, `docs/contracts/076-low-level-correctness-safety-and-protocol-hardening-contract.md`, `docs/contracts/077-dsp-fidelity-semantic-calibration-and-analysis-realism-contract.md`, `docs/contracts/078-rhythm-continuity-failure-containment-and-policy-normalization-contract.md`, `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`

## Summary

Opened the post-audit `g09` generation, added the missing planning front doors,
froze the remediation contracts, and completed the first `g09.002` tranche by
recording the shared plugin-hosting discovery and sandbox capability matrix.

## What Landed

- added `docs/architecture/system-inventory.md`
- added `docs/contracts/contract-index.md`
- added remediation contracts `072` through `079`
- opened `docs/roadmaps/g09/`
- completed `g09.001`
- completed `g09.002` Batch 2.1 by recording the current adapter/host/sandbox
  scaffold posture and the replacement target in contract `072`

## Validation

- `effigy qa:docs` passed
- `effigy qa:northstar` passed
- `effigy validate` remains blocked by pre-existing workspace issues:
  - unresolved imports in `crates/signal-analysis-tonal/src/tests.rs`
  - unused-import warnings in `crates/signal-plugin-clap/src/tests/block_processing.rs`
  - unused-import warnings in `crates/signal-plugin-clap/src/tests/lifecycle.rs`

## Next Task

Start `g09.002` Batch 2.2 by replacing fixture-backed discovery in the adapter
and host scan path with a shared filesystem-backed discovery foundation while
preserving the current runtime-owned receipt boundary.
