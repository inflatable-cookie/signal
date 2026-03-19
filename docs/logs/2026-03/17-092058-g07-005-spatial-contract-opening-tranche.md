# 2026-03-17 - g07.005 spatial contract opening tranche

## Summary

Opened Batch 5.1 of `g07.005` by freezing the first reusable Signal-owned
spatial adapter execution contract in
`docs/contracts/036-spatial-adapter-execution-contract.md`.

The new contract makes spatial execution build on the already-closed
multichannel, sidechain, multi-bus, and complex plugin-I/O seams instead of
letting spatial behavior drift back into host-local pan rules, product mixer
policy, or adapter-private renderer naming.

## Key decisions

- froze bounded shared spatial vocabulary for:
  - adapter class
  - execution mode
  - target environment
  - control family
  - activation policy
  - fallback outcome
- anchored the contract to the Chorus spatial adapter spec and current Signal
  runtime topology surfaces rather than pretending richer execution already
  exists
- kept surround beds, objects, room calibration, and product-local immersive UX
  explicitly deferred to later `g07` milestones

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Next Task

Continue `g07.005` with Batch 5.2 by materializing runtime-owned spatial
adapter execution, target-environment, and fallback receipts across execution,
render, and observation surfaces without reopening host-local or adapter-local
spatial ownership.
