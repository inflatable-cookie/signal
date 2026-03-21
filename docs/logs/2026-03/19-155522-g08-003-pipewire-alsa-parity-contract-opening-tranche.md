# 2026-03-19 - g08.003 Batch 3.1 PipeWire And ALSA Parity Contract Opening

## Summary

Opened `g08.003` by freezing the first runtime-owned PipeWire and ALSA parity
contract for session role, device-claim posture, and stream-policy meaning.

## Delivered

- added `docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md`
- defined the authority hierarchy across live Linux ownership, JACK-native
  coordination, Linux portability, Linux clocking parity, and fault or restart
  boundaries
- froze shared vocabulary for PipeWire and ALSA:
  - session-role parity
  - device-claim parity
  - stream-policy parity
  - guarded parity
- updated the active roadmap, contract index, architecture reference, and
  generation pointers so Batch 3.2 is now the live next step

## Validation

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual Risk

This tranche freezes meaning, not runtime realization. PipeWire node, daemon,
and portal depth and ALSA reservation or duplex policy depth remain deferred
until Batch 3.2 materializes typed parity receipts.

## Next Task

Continue `g08.003` with Batch 3.2 by materializing the first runtime-owned
PipeWire and ALSA session-role, device-claim, and stream-policy parity
receipts, then align stable host-edge export to the same parity model.
