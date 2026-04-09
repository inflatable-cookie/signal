# 024 - g09.012 Host Demo Bootstrap Fix

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.012
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Auto-start next card: no

## Objective

Take the next honest `g09.012` seam by fixing the existing host demo bring-up
path so the `signal-host-local` and `signal-host-server` binaries boot through
an explicit supported demo plugin surface instead of failing immediately on the
deferred CLAP sandbox path.

## Scope

- stay inside the host demo bootstrap path used by the existing host binaries
- fix the bring-up route without widening into CLAP hosting implementation
- keep the fix explicit and demo-oriented rather than hiding unsupported CLAP
  truth
- do not widen into full local-versus-server comparison output design yet

## Steps

1. Freeze the host bootstrap-fix seam from `g09.012` and contract `079`.
2. Route the host demo bring-up path through an explicit supported demo plugin
   surface instead of the implicit CLAP default.
3. Keep the unsupported CLAP truth explicit outside the demo bootstrap path.
4. Record the fix and the updated host-demo posture in the roadmap and demo
   substrate docs.
5. Rerun focused host demo launch validation plus repo health and docs checks.

## Acceptance Criteria

- `cargo run -p signal-host-local` boots successfully on the demo path
- `cargo run -p signal-host-server` boots successfully on the demo path
- the fix does not claim CLAP sandbox support exists where it is still
  deferred
- the batch stays bounded to bring-up and does not widen into a full host
  comparison demo
- focused validation passes

## Evidence Required

- batch log for the next `g09.012` tranche
- validation actually run
- explicit note that host comparison output shaping still remains deferred

## Stop Conditions

- the batch starts implementing real CLAP hosting instead of demo bootstrap
  routing
- the existing host binaries are not actually salvageable as the bootstrap
  surface
- the work turns into a broader host demo design instead of a narrow bring-up
  fix

## Outcome

- `signal-host-local` and `signal-host-server` now provision a temporary
  supported VST3 demo bundle when no explicit host demo override is already set
- the existing host binaries now boot successfully without falling back to the
  deferred CLAP sandbox path
- unsupported CLAP sandbox truth remains unchanged outside the binary bootstrap
  path
- host comparison output shaping and plugin capability browsing both remain
  deferred pending fresh planning judgment

## Validation

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo run -q -p signal-host-local`
- `cargo run -q -p signal-host-server`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is local-versus-server comparison, plugin capability
browsing, or a continued planning pause.
