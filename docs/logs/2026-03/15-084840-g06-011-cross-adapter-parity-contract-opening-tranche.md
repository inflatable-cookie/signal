# 2026-03-15 08:48:40 UTC - g06.011 Cross-Adapter Parity Contract Opening Tranche

## Summary

Opened `g06.011` Batch 11.1 by freezing the first bounded cross-adapter parity
contract across CLAP, VST3, and AU, with Linux plugin support made explicit as
guarded rather than implied.

## Work completed

- added
  `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`
- froze one shared parity vocabulary separating:
  - portable scope
  - format-guarded scope
  - adapter-private scope
  - unsupported or deferred scope
- made Linux plugin breadth explicit in the shared parity matrix:
  - CLAP guarded
  - VST3 guarded
  - AU unsupported
- updated roadmap, contract index, generation pointers, and feature reference
  so Batch 11.2 is now the single active follow-on queue

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred scope

- runtime-owned platform-coverage and unsupported-state receipts still need
  Batch 11.2 depth
- richer cross-adapter event, editor, preset, parameter-tree, and unit-depth
  parity remains later work rather than part of this bounded contract

## Next Task

Continue `g06.011` with Batch 11.2 by aligning discovery, lifecycle, render,
failure, placement, and platform-coverage receipts across CLAP, VST3, and AU
so the frozen parity matrix becomes directly inspectable through runtime,
supervisor, and stable host-edge surfaces.
