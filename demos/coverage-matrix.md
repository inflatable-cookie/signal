# Demo Coverage Matrix

Status: active
Updated: 2026-04-10

## Purpose

Freeze the first repo-owned crate-to-demo inventory for the active Signal
workspace without overclaiming live demo coverage before `g09.012` and
`g09.013` land.

## Current posture

- live official demo manifests:
  - `signal.demo.plugin.sandbox-lifecycle`
  - `signal.demo.runtime.recovery-inspector`
  - `signal.demo.runtime.supervisor-boundary-companion`
  - `signal.demo.host.local-server-compare`
  - `signal.demo.hardware.topology-diagnostics`
  - `signal.demo.macos.au-coreaudio-boundary`
  - `signal.demo.linux.lv2-backend-boundary`
  - `signal.demo.graph.execution-inspector`
  - `signal.demo.dsp.processing-lab`
  - `signal.demo.analysis.feature-inspector`
- shared substrate is frozen under `demos/`
- every active workspace crate is mapped to either a live demo manifest or an
  explicit planned demo surface and milestone

## Live surfaces

- `signal.demo.plugin.sandbox-lifecycle`
  - crates: `signal-plugin-sandbox`, `signal-ipc`
  - launch: `effigy demo:sandbox-lifecycle`
- `signal.demo.runtime.recovery-inspector`
  - crates: `signal-runtime`
  - launch: `effigy demo:runtime-recovery-inspector`
- `signal.demo.runtime.supervisor-boundary-companion`
  - crates: `signal-supervisor-tools`
  - launch: `effigy demo:supervisor-runtime-boundary-companion`
- `signal.demo.host.local-server-compare`
  - crates: `signal-host-local`, `signal-host-server`
  - launch: `effigy demo:local-server-host-comparison`
- `signal.demo.hardware.topology-diagnostics`
  - crates: `signal-hardware`, `signal-hardware-coreaudio`
  - launch: `effigy demo:hardware-topology-diagnostics`
- `signal.demo.macos.au-coreaudio-boundary`
  - crates: `signal-plugin-au`
  - launch: `effigy demo:macos-au-coreaudio-boundary`
- `signal.demo.linux.lv2-backend-boundary`
  - crates: `signal-plugin-lv2`
  - launch: `effigy demo:linux-lv2-and-backend-boundary`
- `signal.demo.graph.execution-inspector`
  - crates: `signal-primitives`, `signal-graph`
  - launch: `effigy demo:graph-execution-inspector`
- `signal.demo.dsp.processing-lab`
  - crates: `signal-dsp`, `signal-dsp-resample`, `signal-dsp-spectral`
  - launch: `effigy demo:dsp-processing-lab`
- `signal.demo.analysis.feature-inspector`
  - crates: `signal-analysis`, `signal-analysis-character`,
    `signal-analysis-embed`, `signal-analysis-loudness`,
    `signal-analysis-rhythm`, `signal-analysis-tonal`
  - launch: `effigy demo:analysis-feature-inspector`

## Planned surfaces

### `g09.015` operator-visible interactive demos

- `signal.demo.plugin.capability-browser`
  - crates: `signal-plugin`, `signal-plugin-vst3`, `signal-plugin-au`,
    `signal-plugin-clap`
  - current blocker: real CLAP discovery and removal of VST3/AU metadata-file
    discovery shims

## Deferred After g09

- none at the current demo-planning layer

## Working rule

- do not mark a crate as live-covered until a manifest in `demos/manifests/`
  claims it
- do not widen the matrix into ad hoc product shells
- if a crate moves milestones or surfaces, update this file and
  `demos/coverage-matrix.json` together

## Next Task

Continue the reopened strict `g09` lane from
`docs/specs/batch-cards/044-g09-015-real-plugin-discovery-gap-burn-down.md`.
