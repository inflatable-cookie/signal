# 2026-03-16 22:06:13 UTC - g07.002 sidechain contract opening tranche

## Summary

Opened `g07.002` by freezing the first reusable sidechain and secondary-input
routing contract.

## Why this tranche matters

`g07.002` needed a real authority line before runtime execution depth could
start. This tranche prevents live routing, offline render, plugin-format
integration, and later multi-bus work from inventing separate sidechain models
or falling back to host-local patch conventions.

## What changed

- added `docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md`
  to freeze sidechain source, target, attachment-policy, and fallback meaning
- recorded the Batch 2.1 outcome in the active `g07.002` roadmap
- rolled shared roadmap, contract, generation-index, and architecture pointers
  forward to Batch 2.2 runtime execution depth
- kept the sidechain scope bounded so multi-bus, complex plugin-I/O, and
  spatial routing work remain explicit later milestones instead of hidden scope

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Residual risk

This tranche freezes routing meaning only. Runtime-owned live and offline
secondary-input receipts, fallback behavior, and consumer-facing proof still
belong to the next `g07.002` batches.
