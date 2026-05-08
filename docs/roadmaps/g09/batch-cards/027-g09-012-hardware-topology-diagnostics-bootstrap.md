# 027 - g09.012 Hardware Topology Diagnostics Bootstrap

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.012
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Auto-start next card: no

## Objective

Take the next honest `g09.012` seam by turning the existing native local-host
and simulated server-host hardware/device outputs into one bounded hardware
topology and diagnostics demo surface.

## Scope

- stay inside the current host binaries and their existing hardware-facing
  output surfaces
- build a repo-owned manifest, operator notes, launch task, and receipt that
  highlight device supervision, external I/O, backend, and endpoint topology
  truth
- use the local host for the native CoreAudio side and the server host for the
  simulated Linux backend side instead of inventing a new executable
- keep plugin capability browsing explicitly deferred instead of blending scan
  browsing into this hardware batch
- do not widen into a host UI shell or a new hardware control surface

## Steps

1. Freeze the bounded hardware diagnostics seam from `g09.012` and contract
   `079`.
2. Add a hardware diagnostics manifest, operator notes, receipt path, and
   Effigy launch task under `demos/`.
3. Implement a repo-owned wrapper that runs the current host binaries, captures
   their existing hardware-facing summary fields, and emits one machine-readable
   receipt.
4. Keep native-versus-simulated and supported-versus-deferred posture explicit
   in the manifest and receipt instead of claiming full hardware breadth.
5. Update the roadmap, coverage matrix, and strict currentness surfaces if the
   batch closes cleanly.

## Acceptance Criteria

- one repo-owned launch surface captures both native and simulated hardware
  posture from the existing host binaries
- the receipt includes device supervision, backend identity, and endpoint or
  stream-state truth from the current host outputs
- `signal-hardware` and `signal-hardware-coreaudio` move to live coverage only
  if the receipt and manifest are actually in place
- the batch stays bounded to hardware topology and diagnostics bootstrap and
  does not widen into plugin browsing or a new host interface
- focused validation passes

## Evidence Required

- batch log for the next `g09.012` tranche
- validation actually run
- explicit note whether plugin capability browsing remains deferred after this
  batch

## Stop Conditions

- the work starts redesigning host binary output instead of wrapping the
  existing hardware-facing fields
- the seam needs fresh planning about what counts as “hardware topology” beyond
  the currently exported host/device truth
- the batch starts implementing plugin capability browsing or a new host UI

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is plugin capability browsing, another bounded
host/runtime/hardware live-demo batch, or a continued planning pause.
