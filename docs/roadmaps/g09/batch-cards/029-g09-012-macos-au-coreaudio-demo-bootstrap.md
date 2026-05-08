# 029 - g09.012 macOS AU CoreAudio Demo Bootstrap

Status: complete
Owner: core-product
Updated: 2026-04-10
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.012
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/073-native-backend-device-truth-and-coreaudio-implementation-contract.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Auto-start next card: no

## Objective

Take the next honest `g09.012` seam by turning the existing macOS AU/CoreAudio
acceptance and boundary-descriptor surfaces into one repo-owned live demo
scenario.

## Scope

- stay inside the current `signal-supervisor-tools` macOS AU/CoreAudio boundary
  descriptor and the existing `effigy acceptance:macos-au-coreaudio-boundary`
  task
- build a repo-owned manifest, operator notes, launch task, and receipt under
  `demos/`
- keep the surface explicitly macOS-specific and do not claim Linux-native or
  cross-platform plugin browsing breadth
- promote the relevant crates to live coverage only if the manifest and receipt
  are actually in place
- do not widen into a new host UI, plugin scan-root design, or generalized
  plugin capability browser

## Steps

1. Freeze the bounded macOS AU/CoreAudio seam from `g09.012` and contracts
   `073` and `079`.
2. Add a macOS AU/CoreAudio manifest, operator notes, receipt path, and Effigy
   launch task under `demos/`.
3. Implement a repo-owned wrapper that runs the current
   `--describe-macos-au-coreaudio-boundary --format=json` descriptor and the
   existing `effigy acceptance:macos-au-coreaudio-boundary` lane, then emits
   one machine-readable receipt.
4. Keep the surface explicitly macOS-specific and explicit about deferred Linux
   and general plugin-browsing breadth.
5. Update the roadmap, coverage matrix, and strict currentness surfaces if the
   batch closes cleanly.

## Acceptance Criteria

- one repo-owned launch surface captures both the machine-readable macOS
  boundary descriptor and the existing acceptance-task posture
- the receipt keeps AU lifecycle plus CoreAudio device truth explicit and does
  not flatten them into a generic plugin demo
- the batch stays bounded to the macOS AU/CoreAudio bootstrap seam
- focused validation passes

## Evidence Required

- batch log for the next `g09.012` tranche
- validation actually run
- explicit note whether plugin capability browsing remains deferred after this
  batch

## Stop Conditions

- the work starts redesigning AU/CoreAudio runtime behavior instead of wrapping
  the current descriptor and acceptance surfaces
- the seam needs fresh planning about broader cross-platform plugin browsing
  before the demo can be executed honestly
- the batch starts implementing a generalized plugin capability browser or a
  new host UI

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is plugin capability browsing, Linux-native backend/LV2
demo coverage, or a continued planning pause.
