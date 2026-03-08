# Trust-Edge Package Shell Expansion

Date: 2026-03-08
Owner: core-product

## Summary

Expanded the initial Signal Rust workspace shell with the first trust-edge
package set and wired the local/server host shells to those new boundaries.

## Work completed

- added real workspace members for:
  - `signal-ipc`
  - `signal-plugin`
  - `signal-plugin-clap`
  - `signal-plugin-sandbox`
  - `signal-hardware`
  - `signal-hardware-coreaudio`
- added minimal manifests and source shells for each new package
- updated `signal-host-local` and `signal-host-server` to reference the plugin
  and hardware shells directly so ownership boundaries are visible in code
- updated Signal-owned docs:
  - package map now reflects the current workspace state
  - `g01.004` now tracks the trust-edge expansion batch
  - docs entry points now point at the active trust-edge milestone
- marked the earlier naming and workspace-shell milestones complete once their
  outputs were clearly frozen

## Validation

- `cargo check --workspace`
- `git diff --check`

## Notes

The new crates are intentionally thin. Their job in this batch is to freeze
workspace ownership and dependency shape, not to implement real CLAP hosting,
CoreAudio I/O, or sandbox shared-memory transport yet.

## Next Task

Map the runtime-host interface contract and sandbox protocol onto real Rust
modules inside `signal-runtime`, `signal-host-local`, `signal-host-server`,
`signal-plugin`, and `signal-hardware`, then define the first CLAP sandbox
shared-memory header/state-machine types.
