# 073 Native Backend Device Truth And CoreAudio Implementation Contract

Status: draft
Owner: core-product
Updated: 2026-04-08
Related contracts: `docs/contracts/006-runtime-hardware-portability-and-clock-domain-contract.md`, `docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md`, `docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`
Related architecture: `docs/architecture/system-architecture.md`

## Purpose

Freeze the contract for replacing simulated native-backend placeholders with
real device enumeration, claim, stream-policy, and diagnostics behavior,
starting with CoreAudio.

## Authority hierarchy

1. `signal-hardware` owns backend-neutral device, stream, and diagnostics
   vocabulary.
2. `signal-runtime` owns runtime-visible hardware and host-I/O summaries.
3. native backend crates such as `signal-hardware-coreaudio` own OS-specific
   realization.
4. host crates may orchestrate runtime bring-up but must not invent their own
   device identity or diagnostics taxonomy.

## Required shared guarantees

- CoreAudio device enumeration must produce real device and endpoint identity.
- runtime-visible diagnostics must distinguish unavailable, degraded, and
  healthy backend states without synthetic default-device shortcuts.
- backend-native stream-policy and clock-domain detail must map into existing
  shared hardware receipts rather than parallel macOS-only DTOs.

## Rules

- Simulated-device baselines remain test-only once native realization lands.
- CoreAudio device and stream failures must map into shared runtime fault and
  diagnostics receipts.
- Native backend capability gaps must be explicit in receipts, not hidden by
  fake default devices.

## Required proof surfaces

- focused CoreAudio enumeration and diagnostics tests
- stable host-edge proofs over runtime-owned hardware summaries
- at least one interactive hardware demo path under contract `079`

## Next Task

Use this contract for the `g09` AU/CoreAudio milestone and any later native
backend realization beyond Linux.
