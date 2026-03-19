# 2026-03-17 - g07.009 Linux backend contract opening tranche

## Summary

Opened Batch 9.1 of `g07.009` by freezing the first runtime-owned Linux audio
backend portability boundary across ALSA, JACK, and PipeWire.

This tranche establishes one shared Linux backend vocabulary before runtime or
host implementation work widens into backend-specific baselines.

## Key changes

- added the new contract
  `docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md`
- fixed the authority chain so Linux backend identity, supervision posture,
  clocking composition, and fallback meaning remain Signal-owned rather than
  host-local or backend-private
- froze the guarded-first ALSA/JACK/PipeWire matrix and explicitly deferred
  distro-specific, daemon-specific, and richer backend-native session detail
- rolled roadmap, contract, and architecture references forward so Batch 9.2
  can focus on real backend baselines

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Residual risk

This tranche freezes meaning only. ALSA, JACK, and PipeWire still do not have
shared runtime baselines or proof depth here yet, and later `g07.010` still
owns deeper Linux clocking, duplex, and endpoint-topology parity.

## Next Task

Continue `g07.009` with Batch 9.2 by materializing the first runtime-owned
Linux backend baselines and aligned diagnostics across ALSA, JACK, and
PipeWire without reopening backend-private lifecycle or health ownership.
