# 026 - g09.012 CLAP Host Sandbox Fix

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.012
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Auto-start next card: no

## Objective

Take the real blocked `g09.012` seam by replacing the current explicit CLAP
unsupported-path error in `signal-host-local` and `signal-host-server` with one
bounded real CLAP host sandbox integration path.

## Scope

- stay inside the existing host-side CLAP sandbox ensure and restart paths
- reuse the already existing CLAP adapter, lifecycle harness, prepare-plan, and
  block-protocol surfaces instead of inventing a new demo-only route
- implement the narrowest honest host integration slice needed to stop treating
  CLAP as unsupported on the host sandbox path
- update the existing public parity/gap proofs that currently assert the old
  unsupported behavior
- do not widen into a full host comparison demo, plugin browser, or broader
  CLAP feature expansion beyond the host sandbox path

## Steps

1. Freeze the CLAP host-sandbox gap from `g09.012` and contract `079`.
2. Add a bounded host-side CLAP sandbox session path aligned with the existing
   AU, VST3, and LV2 prepared/broker shells where appropriate.
3. Route local and server host `ensure_plugin_sandbox(...)` and restart through
   that CLAP path instead of returning the current unsupported error.
4. Update the public cross-adapter parity proofs so they assert real CLAP host
   lifecycle truth instead of the old gap receipt.
5. Record the real CLAP host-demo posture in the roadmap and strict surfaces,
   then reassess whether host comparison is finally the next honest demo seam.

## Acceptance Criteria

- `PluginFormat::Clap` no longer returns the current unsupported-path error in
  the local and server host sandbox ensure path
- the fix reuses the existing CLAP adapter and lifecycle surfaces instead of a
  demo-only workaround
- the public host-edge parity proofs stop asserting the CLAP gap and instead
  assert bounded CLAP lifecycle truth
- the batch stays inside host-side CLAP sandbox integration and does not widen
  into broader demo design
- focused validation passes

## Evidence Required

- batch log for the next `g09.012` tranche
- validation actually run
- explicit note whether host comparison is now truly ready or still blocked by
  another non-demo seam

## Stop Conditions

- the work starts implementing unrelated CLAP feature expansion beyond the host
  sandbox path
- the existing CLAP adapter/harness surfaces are not actually sufficient for a
  bounded host integration slice
- the batch turns into host comparison output design instead of CLAP path
  repair

## Next Task

Return to
`docs/roadmaps/g09/batch-cards/025-g09-012-local-server-host-comparison-bootstrap.md`
now that the host comparison surface is no longer blocked by the CLAP gap.
