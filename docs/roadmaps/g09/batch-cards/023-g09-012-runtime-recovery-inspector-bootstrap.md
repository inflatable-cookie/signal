# 023 - g09.012 Runtime Recovery Inspector Bootstrap

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.012
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Auto-start next card: no

## Objective

Take the next honest `g09.012` live-demo seam by turning the existing
`signal-runtime` supervisor report example into one official runtime recovery
inspector surface with a manifest, repo-owned launch task, and operator
evidence.

## Scope

- stay inside the runtime recovery inspector seam and the minimum shared demo
  files needed to make it an official live demo
- use the existing `signal-runtime --example supervisor_report_demo` surface
  rather than inventing a new runtime shell
- add one manifest, one launch task, and one operator-evidence path
- keep host comparison, plugin capability browsing, and hardware demos deferred

## Steps

1. Freeze the runtime recovery inspector bootstrap seam from `g09.012` and
   contract `079`.
2. Add one official live demo manifest for the runtime recovery inspector.
3. Add the repo-owned launch task and the minimum receipt/operator-note files
   that make the surface inspectable.
4. Record the bootstrap surface in the roadmap and demo substrate docs.
5. Rerun focused repo health and docs validation plus the new launch task.

## Acceptance Criteria

- one official runtime recovery inspector manifest exists under
  `demos/manifests/`
- the demo is runnable through a repo-owned task
- operator evidence paths are explicit and colocated
- the work does not widen into host comparison, plugin browsing, or hardware
  demo breadth
- focused validation passes

## Evidence Required

- batch log for the next `g09.012` tranche
- validation actually run
- explicit note that host comparison, plugin capability browsing, and hardware
  demos remain deferred to later `g09.012+` batches

## Stop Conditions

- the batch starts implementing local-versus-server host comparison
- the existing runtime example is not actually sufficient for an honest
  bootstrap demo surface
- the work turns into new CLI or app-shell design instead of demo substrate
  capture

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether `g09.012`
needs a bounded bootstrap-fix card first or should remain paused until a clean
live-demo seam is genuinely ready.
