# Demo Program

Status: active
Updated: 2026-04-14

## Purpose

This folder is the repo-owned demo program for Signal.

It does not replace crate code. It defines the authority layer that turns demo
commands into inspectable proof instead of loose examples, one-off scripts, or
operator memory.

## Layout

- `manifest.schema.json`
  - machine-readable schema for every official demo manifest
- `templates/`
  - manifest examples and starter shapes for new demo surfaces
- `manifests/`
  - machine-readable manifests for official demo binaries or scenario bundles
- `coverage-matrix.json`
  - machine-readable crate-to-demo coverage inventory for the active workspace
- `coverage-matrix.md`
  - human-readable view of the same coverage inventory
- `effigy.demos.toml`
  - shared demo task shim plus local manifest-import root for the demo registry
- `effigy.demos.operator-views.toml`
  - headless operator-view demo records
- `effigy.demos.platform-boundaries.toml`
  - headless platform-boundary demo records
- `effigy.demos.interactive.toml`
  - interactive demo records
- `receipts/`
  - repo-owned proof artifacts for official demos
- `scenarios/`
  - operator-facing notes for shared scenario bundles when human checks matter
- `scripts/lib/`
  - shared Bun/TS runner helpers for headless and interactive demo surfaces
- `scripts/`
  - Bun/TS demo runner entrypoints; the runner name should match the manifest
    surface name

## Proof Policy

- manifests, scenarios, coverage inventory, and receipts are the authoritative
  demo layer
- receipt JSON and rendered `.view.html` companions are repo-owned proof, not
  disposable build trash
- transient runtime logs, local scan noise, and ad hoc browser sessions are not
  part of the repo-owned proof layer
- if a demo should stay reproducible in CI, its receipt path and rendered
  companion should remain tracked

## Current Shape

- registry files are split by concern instead of accumulating in one giant
  manifest:
  - operator views
  - platform boundaries
  - interactive surfaces
- Bun/TS now owns all live demo runners
- the plugin capability browser is no longer a Python exception
- `receipts/` remains flat on purpose because each live demo has one canonical
  proof pair today:
  - `<surface>.receipt.json`
  - `<surface>.view.html`
- historical `g09` planning and closeout docs may still mention older runner
  paths; the live authority is the current file layout under `demos/`
- if history and live surfaces disagree on runner language or file names, trust
  the current demo registry and `demos/scripts/`

## Program Shape

- the authoritative declaration for an official demo is a manifest in
  `demos/manifests/`
- a manifest may point at:
  - `cargo run -p <crate> --example <name>`
  - `cargo run -p <crate> --bin <name>`
  - a future repo-owned wrapper command exposed through Effigy
- existing crate `examples/` are not official demo surfaces until a manifest
  claims them
- scenario identity belongs in the manifest, not only in code comments or
  operator memory
- official active demos should prefer an `effigy-task` launch owner surface once
  they are promoted beyond placeholder or template status
- cargo example or cargo bin launch commands are acceptable for planned
  substrate and bootstrap stages while the later launch posture is still being
  installed

## Naming Rules

- manifest files use kebab-case:
  `<domain>-<surface>.demo.json`
- manifest ids use the stable namespace:
  `signal.demo.<domain>.<surface>`
- scenario ids use the stable namespace:
  `signal.demo.<domain>.<surface>.<scenario>`

## Grouping Rule

- prefer a shared domain demo when one operator workflow naturally proves
  multiple closely related crates together
- use a dedicated demo only when a crate has a distinct operator workflow,
  unique external prerequisites, or would be hidden by a shared surface

## Evidence Convention

- every official demo manifest must name:
  - a machine-readable receipt path
  - an operator-notes path
  - a shared capture mode
- receipts belong in `demos/receipts/` for live demos or `demos/templates/`
  for substrate examples
- operator notes belong in `demos/scenarios/` for live demos or
  `demos/templates/` for substrate examples
- receipts should capture the launched command, scenario identity, run status,
  and explicit operator-check outcomes
- operator notes should capture the human checks that are meaningful but not
  machine-readable

## Launch Convention

- official demo launches must be repo-owned and explicit in the manifest
- preferred launch owner surface:
  - `effigy-task`
- temporary bootstrap owner surfaces:
  - `cargo-example`
  - `cargo-bin`
- a later demo batch should promote active demos from cargo-owned launch
  commands to Effigy-owned launch tasks as the substrate matures
- the first official live demo surfaces are:
  - `signal.demo.plugin.sandbox-lifecycle`
  - `signal.demo.runtime.recovery-inspector`
  - `signal.demo.runtime.supervisor-boundary-companion`
  - `signal.demo.host.local-server-compare`
  - `signal.demo.hardware.topology-diagnostics`
  - `signal.demo.graph.execution-inspector`
  - `signal.demo.dsp.processing-lab`

## Current Boundary

- the demo surface is now fully Bun/TS-backed
- Effigy owns discovery and launch through the demo registry
- the current live set stays intentionally small and explicit rather than
  widening into product shells or ad hoc utility scripts

## Next Task

Reassess whether another live demo-surface cleanup is actually needed before
editing historical `g09` material. Prefer changing current authority files over
rewriting archival closeout records.
