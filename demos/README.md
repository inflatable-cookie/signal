# Demo Program

Status: active
Updated: 2026-04-09

## Purpose

This folder is the shared substrate for repo-owned Signal demo programs.

It does not replace crate code. It defines the authority layer that turns demo
commands into inspectable proof instead of loose examples.

## Layout

- `manifest.schema.json`
  - machine-readable schema for every official demo manifest
- `templates/`
  - manifest examples and starter shapes for new demo surfaces
- `manifests/`
  - future machine-readable manifests for concrete demo binaries or scenario
    bundles
- `scenarios/`
  - future operator-facing notes for shared scenario bundles when human checks
    matter

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

## Current Boundary

- this substrate pack freezes the program shape only
- domain demos and coverage matrices remain deferred to `g09.012+`

## Next Task

Implement the first concrete manifests and launch/evidence conventions on top
of this substrate.
