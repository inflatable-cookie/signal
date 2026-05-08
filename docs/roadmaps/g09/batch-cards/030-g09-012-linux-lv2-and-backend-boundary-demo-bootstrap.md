# 030 - g09.012 Linux LV2 And Backend Boundary Demo Bootstrap

Status: complete
Owner: core-product
Updated: 2026-04-10
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.012
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md, docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Auto-start next card: no

## Objective

Take the next honest `g09.012` seam by turning the existing Linux LV2
execution and Linux audio-backend acceptance-plus-descriptor surfaces into one
repo-owned live demo scenario.

## Scope

- stay inside the current `signal-supervisor-tools` Linux boundary descriptor
  commands and the existing `effigy acceptance:linux-lv2-execution-boundary`
  and `effigy acceptance:linux-audio-backend-boundary` tasks
- build one repo-owned manifest, operator notes file, launch task, and receipt
  under `demos/`
- keep the surface explicitly Linux-focused and do not claim generalized plugin
  browsing, cross-platform parity browsing, or live Linux device ownership
- promote the relevant crates to live coverage only if the manifest and receipt
  are actually in place
- do not widen into scan-root design, a generalized plugin browser, or host UI
  redesign

## Steps

1. Freeze the bounded Linux seam from contracts `040`, `055`, and `079`.
2. Add a Linux LV2/backend manifest, operator notes, receipt path, and Effigy
   launch task under `demos/`.
3. Implement a repo-owned wrapper that runs the current
   `--describe-linux-lv2-execution-boundary --format=json` and
   `--describe-linux-audio-backend-boundary --format=json` descriptor commands
   plus the existing acceptance tasks, then emits one machine-readable receipt.
4. Keep the surface explicit about Linux-only scope, residual backend limits,
   and the still-deferred plugin capability browser.
5. Update the roadmap, coverage matrix, and strict currentness surfaces if the
   batch closes cleanly.

## Acceptance Criteria

- one repo-owned launch surface captures both the Linux LV2 execution boundary
  and Linux audio-backend boundary without flattening them into a generic
  plugin demo
- the receipt keeps bounded LV2 broker-execution truth and Linux backend
  identity truth explicit
- the batch stays bounded to the Linux boundary bootstrap seam
- focused validation passes

## Evidence Required

- batch log for the next `g09.012` tranche
- validation actually run
- explicit note whether plugin capability browsing remains deferred after this
  batch

## Stop Conditions

- the work starts designing demo-owned plugin scan roots or browse posture
  instead of wrapping the current Linux boundary surfaces
- the seam needs fresh planning about generalized Linux live ownership or
  distro-wide coverage before the demo can be executed honestly
- the batch starts implementing a generalized plugin browser or a new host UI

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether
plugin capability browsing is now tightly batch-cardable or whether `g09.012`
should pause and hand off into `g09.013`.
