# 022 - g09.012 Sandbox Lifecycle Demo Bootstrap

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.012
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Auto-start next card: no

## Objective

Take the first honest `g09.012` live-demo seam by turning the existing sandbox
broker binary into one official demo surface with a manifest, repo-owned launch
task, and operator evidence placeholders.

## Scope

- stay inside the sandbox lifecycle surface and the minimum shared demo files
  needed to make it an official live demo
- use the existing `signal-plugin-sandbox` binary rather than inventing a new
  app shell
- add one manifest, one launch task, and one operator-evidence path
- do not widen into plugin capability browsing, host comparison, runtime
  recovery, or hardware demos yet

## Steps

1. Freeze the bootstrap sandbox-lifecycle seam from `g09.012` and contract
   `079`.
2. Add one official live demo manifest for the sandbox broker lifecycle.
3. Add the repo-owned launch task and the minimum receipt/operator-note files
   that make the surface inspectable.
4. Record the bootstrap surface in the roadmap and demo substrate docs.
5. Rerun focused repo health and docs validation plus the new launch task.

## Acceptance Criteria

- one official sandbox lifecycle demo manifest exists under `demos/manifests/`
- the demo is runnable through a repo-owned task
- operator evidence paths are explicit and colocated
- the work does not widen into the rest of the plugin or host demo suite
- focused validation passes

## Evidence Required

- batch log for the next `g09.012` tranche
- validation actually run
- explicit note that plugin capability browsing and host/runtime/hardware demos
  remain deferred to later `g09.012+` batches

## Stop Conditions

- the batch starts implementing a broader plugin or host demo suite
- the existing sandbox binary is not actually sufficient for an honest
  bootstrap demo surface
- the work turns into new CLI/product-shell design instead of demo substrate
  capture

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is plugin capability browsing, runtime/host demo
bootstrap, or a planning pause before creating another ready card.
