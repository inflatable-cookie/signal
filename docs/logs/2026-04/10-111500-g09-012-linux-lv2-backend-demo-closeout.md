# 10-111500 - g09.012 Linux LV2 And Backend Demo Closeout

Status: complete
Owner: core-product
Updated: 2026-04-10
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/specs/batch-cards/030-g09-012-linux-lv2-and-backend-boundary-demo-bootstrap.md

## Summary

Completed the bounded Linux-specific `g09.012` demo batch by wrapping the
existing Linux LV2 execution and Linux audio-backend boundary surfaces into
one repo-owned live demo manifest, task, operator notes file, and receipt.

## Delivered

- added the Linux demo manifest at
  `demos/manifests/linux-lv2-backend-boundary.demo.json`
- added the operator notes at
  `demos/scenarios/linux-lv2-backend-boundary.default.md`
- added the repo-owned wrapper at
  `demos/scripts/run_linux_lv2_and_backend_boundary_demo.py`
- generated the machine-readable receipt at
  `demos/receipts/linux-lv2-backend-boundary.receipt.json`
- added `effigy demo:linux-lv2-and-backend-boundary` in `effigy.toml`
- promoted `signal-plugin-lv2` to live coverage in the demo coverage matrix
- repaired the stale `acceptance:linux-audio-backend-boundary` host-proof
  command so it now targets the real focused server host-edge test binary
  instead of the broken unfocused crate-level invocation

## Validation

- `effigy acceptance:linux-audio-backend-boundary`
- `effigy demo:linux-lv2-and-backend-boundary`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Deferred

- plugin capability browsing remains deferred because owned scan-root and
  browse-posture decisions still are not frozen tightly enough for a ready
  execution card
- this Linux demo proves bounded LV2 execution and Linux backend identity
  truth, not generalized plugin browsing or live Linux device ownership

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether plugin
capability browsing is now tightly batch-cardable or whether `g09.012` should
pause and hand off into `g09.013`.
