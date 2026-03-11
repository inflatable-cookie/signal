# Roadmap g01.008: Device-Backed Host Audio I/O and Diagnostics Baseline

Status: queued
Owner: core-product
Created: 2026-03-10
Depends on: g01.007
Vision tags: RT, IO, RES
Target envelope: connect the now-real runtime engine path to actual host-side
device execution and diagnostics so Signal can run meaningful local audio I/O
without pushing hardware policy into generic runtime or DSP crates.
The exercised engine path should already look like the beginning of a real
node-oriented mixer rather than a permanently flattened stereo callback.

## Problem

Once runtime owns transport and engine processing, the next risk is leaving
hardware execution as a vague shell for too long. Without a real host/device
baseline:

1. runtime work stays trapped in synthetic block loops,
2. device, callback, and underrun behavior will eventually arrive as a rushed
   trust-edge retrofit,
3. Loophole and other consumers will have no reliable reference host for local
   embedded execution,
4. future console-node and track-lane mixer work will still have no proof that
   host execution can carry richer graph topology safely.

## Goals

- define stable host/device contracts in `signal-hardware`
- implement the first meaningful local audio-device path through
  `signal-hardware-coreaudio` and `signal-host-local`
- surface device/runtime diagnostics such as callback timing, xruns, and device
  loss through host and shared runtime reports
- keep audio-device ownership at the host edge rather than inside generic
  runtime crates
- exercise a credible node/lane/bus engine shape through the host path instead
  of proving only a permanently flattened output path

## Non-Goals

- shipping a full cross-platform hardware matrix in this batch
- building browser or remote audio transport here
- implementing full production MIDI/editor tooling in this milestone

## Execution Plan

### 008.1 Hardware contract freeze

- [ ] define the host/device contract for enumeration, stream configuration,
      format negotiation, and lifecycle ownership
- [ ] establish test doubles or simulation seams so host logic can be exercised
      without requiring physical hardware for every validation step
- [ ] make diagnostics contracts explicit for xruns, callback overruns, device
      disappearance, and restart attempts

### 008.2 Local host execution path

- [ ] implement the first concrete CoreAudio-backed stream path at the trust
      edge with explicit start/stop/error behavior
- [ ] connect `signal-host-local` to runtime block processing with a clear audio
      callback or pump boundary
- [ ] add bounded data-transfer rules between host/device buffers and runtime
      buffers so realtime safety remains inspectable
- [ ] prove the host path can run at least one node-oriented mixer shape such as
      track lane to bus to output or console-node to output without reshaping
      generic runtime contracts

### 008.3 Diagnostics and failure handling

- [ ] expose host/device diagnostics through shared runtime reports and any
      host-local summaries that remain justified
- [ ] handle restart and device-loss scenarios without bypassing runtime control
      state
- [ ] validate the local host path with smoke scenarios that include device
      startup, steady-state processing, and fault/degraded behavior
- [ ] include at least one topology-aware validation scenario so diagnostics are
      known to remain meaningful once richer mixer graphs are attached

## Acceptance Signals

1. Signal has one meaningful local audio-device execution path that exercises
   the runtime engine for real.
2. Hardware and callback concerns remain at the host edge rather than leaking
   into generic DSP or runtime packages.
3. Diagnostics cover the first real-world device/runtime failure modes instead
   of only synthetic lifecycle state.

## Risks and Mitigations

- Risk: device integration starts to dictate generic runtime APIs.
- Mitigation: keep `signal-hardware*` and host assemblies as the only owners of
  backend-specific device semantics.
- Risk: hardware work becomes impossible to validate without one machine setup.
- Mitigation: keep contract tests and simulation seams alongside the real local
  host path, then use hardware smoke checks as an additional gate rather than
  the only evidence.

## Evidence Requirements

- [ ] meaningful host/device batches logged under `docs/logs/YYYY-MM/`
- [ ] closure evidence must separate simulated validation from real hardware
      smoke checks
- [ ] any backend-specific compromises recorded explicitly with ownership notes

## Next Task

Open `g01.009` once the runtime engine can run against a real device-backed host
path and the trust-edge device boundary is stable enough to integrate plugin
processing without reworking hardware ownership.
