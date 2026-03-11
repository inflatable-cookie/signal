---
title: g01.009 clap discovery and instance control closure
status: complete
owner: core-product
created: 2026-03-11
tags: [signal, g01, plugin, clap, sandbox]
---

# Summary

Closed the remaining `009.2` CLAP discovery/control gap by replacing the
fixture-only load path with a concrete CLAP-side discovery and instance-control
surface in `signal-plugin-clap`.

## What changed

- added concrete CLAP catalog/control types:
  - `ClapDiscoveredPluginType`
  - `ClapInstanceControlSurface`
- taught `ClapPluginHostAdapter` to:
  - discover supported CLAP plugin types
  - instantiate concrete instance-control surfaces from discovered metadata
- replaced the old synthesized descriptor path in the sandbox lifecycle harness
  with discovered descriptor metadata and explicit contract checks for:
  - load
  - create
  - prepare
  - activate
  - reset
  - destroy
- kept the generic `plugin:clap:*` fallback inside the local CLAP catalog while
  making truly unsupported non-CLAP types fail explicitly
- updated the sandbox example to print the discovered descriptor identity from
  `LoadPluginTypeResponse`

## New evidence

- adapter-level discovery test for concrete CLAP metadata
- lifecycle test for unsupported plugin-type rejection
- lifecycle test for prepare-time block-size contract rejection
- sandbox example now runs through the discovered descriptor path and reports:
  - `plugin:clap:sandbox`
  - `Signal Sandbox CLAP Plugin`
  - `clap`

## Validation

- `cargo test -p signal-plugin-clap`
- `cargo check -p signal-plugin-sandbox`
- `cargo run -q -p signal-plugin-sandbox`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`
- touched-file `git diff --check`

## Outcome

`009.2` is now complete. The remaining open work for `g01.009` is no longer
descriptor/control freeze; it is the first real plugin-backed graph/runtime
execution seam in `009.3`.

## Next

Attach a plugin-backed node to the graph/runtime seam with explicit lifecycle,
latency, tail, and bypass behavior, then prove one real sandboxed plugin
execution path runs through engine processing without collapsing plugin timing
and fault semantics into host-local policy.
