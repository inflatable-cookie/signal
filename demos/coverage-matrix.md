# Demo Coverage Matrix

Status: active
Updated: 2026-04-09

## Purpose

Freeze the first repo-owned crate-to-demo inventory for the active Signal
workspace without overclaiming live demo coverage before `g09.012` and
`g09.013` land.

## Current posture

- live official demo manifests:
  - `signal.demo.plugin.sandbox-lifecycle`
  - `signal.demo.runtime.recovery-inspector`
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
- `signal.demo.host.local-server-compare`
  - crates: `signal-host-local`, `signal-host-server`
  - launch: `effigy demo:local-server-host-comparison`
- `signal.demo.hardware.topology-diagnostics`
  - crates: `signal-hardware`, `signal-hardware-coreaudio`
  - launch: `effigy demo:hardware-topology-diagnostics`

## Planned surfaces

### `g09.012` runtime, host, plugin, and hardware suite

- `signal.demo.plugin.capability-browser`
  - crates: `signal-plugin`, `signal-plugin-vst3`, `signal-plugin-au`,
    `signal-plugin-lv2`, `signal-plugin-clap`
- `signal.demo.runtime.recovery-inspector`
  - remaining deferred crate: `signal-supervisor-tools`
  - note: still deferred until a live supervisor-tools-owned or shared
    host/runtime inspector surface exists

### `g09.013` DSP, graph, and analysis suite

- `signal.demo.graph.execution-inspector`
  - crates: `signal-primitives`, `signal-graph`
- `signal.demo.dsp.processing-lab`
  - crates: `signal-dsp`, `signal-dsp-resample`, `signal-dsp-spectral`
- `signal.demo.analysis.feature-inspector`
  - crates: `signal-analysis`, `signal-analysis-character`,
    `signal-analysis-embed`, `signal-analysis-loudness`,
    `signal-analysis-rhythm`, `signal-analysis-tonal`

## Working rule

- do not mark a crate as live-covered until a manifest in `demos/manifests/`
  claims it
- do not widen the matrix into ad hoc product shells
- if a crate moves milestones or surfaces, update this file and
  `demos/coverage-matrix.json` together

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is plugin capability browsing, another bounded
host/runtime/hardware live-demo batch, or a continued planning pause.
