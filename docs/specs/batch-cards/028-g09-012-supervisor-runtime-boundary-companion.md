# 028 - g09.012 Supervisor Runtime Boundary Companion

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.012
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/002-supervisor-export-schema-and-report-boundary.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Auto-start next card: no

## Objective

Take the next honest `g09.012` seam by extending the existing runtime recovery
inspector family with a repo-owned `signal-supervisor-tools` companion surface,
so the remaining deferred runtime crate is live-covered through a real boundary
descriptor instead of staying planned-only.

## Scope

- stay inside the already existing `signal-supervisor-tools` boundary-descriptor
  CLI surfaces
- pair one or more existing runtime-focused supervisor descriptors with the
  existing runtime inspector family instead of inventing a new runtime or host
  executable
- build a repo-owned manifest, operator notes, launch task, and receipt that
  capture machine-readable supervisor-owned runtime boundary truth
- promote `signal-supervisor-tools` to live coverage only if the companion
  manifest and receipt are actually in place
- do not widen into plugin browsing, host comparison changes, or a new product
  shell

## Steps

1. Freeze the bounded `signal-supervisor-tools` companion seam from `g09.012`
   and contracts `002` and `079`.
2. Add a supervisor runtime-boundary companion manifest, operator notes,
   receipt path, and Effigy launch task under `demos/`.
3. Implement a repo-owned wrapper that runs one or more existing
   `signal-supervisor-tools` runtime-boundary descriptor commands and emits one
   machine-readable receipt.
4. Keep the relationship to the existing runtime recovery inspector explicit in
   the manifest and receipt instead of pretending this is a full replacement
   for runtime example execution.
5. Update the roadmap, coverage matrix, and strict currentness surfaces if the
   batch closes cleanly.

## Acceptance Criteria

- one repo-owned launch surface captures `signal-supervisor-tools`
  machine-readable runtime boundary truth through existing descriptor commands
- the receipt keeps the supervisor-owned boundary posture explicit and ties it
  to the existing runtime inspector family instead of inventing a separate
  product shell
- `signal-supervisor-tools` moves to live coverage only if the manifest and
  receipt actually exist
- the batch stays bounded to the supervisor runtime companion seam and does not
  widen into plugin capability browsing
- focused validation passes

## Evidence Required

- batch log for the next `g09.012` tranche
- validation actually run
- explicit note whether plugin capability browsing remains deferred after this
  batch

## Stop Conditions

- the work starts redesigning the supervisor CLI instead of wrapping existing
  descriptor commands
- the seam needs fresh planning about a broader runtime/host demo family beyond
  the existing runtime inspector and supervisor boundary surfaces
- the batch starts implementing plugin capability browsing or a new host UI

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is plugin capability browsing, another bounded
host/runtime/hardware live-demo batch, or a continued planning pause.
