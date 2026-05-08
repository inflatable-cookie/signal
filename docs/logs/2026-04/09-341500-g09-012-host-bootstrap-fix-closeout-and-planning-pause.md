# 09-341500 - g09.012 Host Bootstrap Fix Closeout And Planning Pause

Status: complete
Owner: core-product
Updated: 2026-04-09
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/roadmaps/g09/batch-cards/024-g09-012-host-demo-bootstrap-fix.md

## Summary

Closed the bounded host demo bootstrap-fix batch by routing the existing local
and server host binaries through an explicit supported demo VST3 surface when
no host demo override is already set.

## Implementation

- added a temporary supported VST3 demo bundle bootstrap helper in
  `signal-host-local` and `signal-host-server`
- updated both host binaries to install that bootstrap helper before
  `boot_default()`
- left the deferred CLAP unsupported-path truth unchanged outside the binary
  bootstrap route
- updated the `g09.012` roadmap and demo coverage matrix to reflect that host
  bootstrap is no longer blocked while host comparison itself remains deferred

## Validation

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo run -q -p signal-host-local`
- `cargo run -q -p signal-host-server`
- `effigy health`

## Notes

- both binaries now boot successfully without an explicit demo env override
- this batch does not yet create a host comparison manifest, receipt, or
  comparison-oriented output surface
- plugin capability browsing still wants fresh planning judgment around a
  demo-owned scan-root surface

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is local-versus-server comparison, plugin capability
browsing, or a continued planning pause.
