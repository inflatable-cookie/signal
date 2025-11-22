# Signal Implementation Plan

This document describes how to implement Signal as a C++20 audio engine process:

- project structure and build system,
- process and threading model,
- IPC integration with Pulse,
- domain structure (engine, transport, graph, etc.),
- real-time audio engine integration,
- testing and validation approach,
- initial milestones.

It is the **"how" companion** to the architecture docs for Signal and IPC, and
should be kept in sync as we refine implementation details.

For architecture documentation, see:
- [Signal README](../README.md) — High-level Signal overview
- [IPC Envelope Spec](https://github.com/infinite-loop-audio/loophole-chorus/blob/main/docs/specs/ipc/envelope.md) — IPC envelope structure
- [Signal Engine Domain](https://github.com/infinite-loop-audio/loophole-chorus/blob/main/docs/specs/ipc/signal/engine.md) — Engine domain IPC spec

---

## 1. Scope & Role of Signal

Signal is the **real-time audio engine** for Loophole:

- Owns the audio processing graph and timebase.
- Hosts plugins (VST3, AU, CLAP – exact set to be finalised later).
- Manages hardware audio I/O, buffers, and low-latency scheduling.
- Communicates with Pulse via the Chorus IPC envelope format.

Pulse is the authoritative data model and session state server.
Signal is a separate process focused on real-time constraints.

---

## 2. Language, Toolchain & Build

- **Language:** Modern C++ (C++20 or later).
- **Build system:** CMake.
- **Runtime model:**
  - Separate process, managed by Pulse (and/or Aura for dev).
  - Strict separation between real-time audio threads and control/IPC threads.
- **Dependencies (initial skeleton):**
  - Standard library only, plus a minimal test framework (e.g. Catch2).
  - No heavy DSP or plugin frameworks yet – those will be added in later milestones.

---

## 3. High-Level Architecture

### 3.1 Core Components

- `SignalApp`
  - Top-level application object.
  - Owns lifecycle: init, run (if needed), shutdown.
  - Owns:
    - IPC server (control thread / event loop).
    - Engine host (audio graph & realtime scheduling – stubbed initially).

- `IpcServer` / `IpcRouter`
  - Listens for JSON IPC envelopes from Pulse using the Chorus envelope spec.
  - Decodes envelopes into an internal `Envelope` struct.
  - Routes by `domain` to domain handlers.
  - For the skeleton, this can be a stub with no actual networking, or a simple TCP server that logs messages.

- `EngineHost` (stub initially)
  - Placeholder for the real-time engine:
    - Audio graph
    - Hardware I/O
    - Timebase
  - For now, only initialises and shuts down cleanly.

### 3.2 Domain Handlers (Stub Phase)

- `engine` domain:
  - Commands (skeleton only): `start`, `stop`, `reset`, `shutdown`.
  - For now, just log and update an in-memory state enum.

- `transport` domain:
  - Commands (skeleton only): `play`, `stop`, `seek`, maybe `setLoop`.
  - For now, only update a transport state struct and log.

Later milestones will add more domains:
- `plugin`, `hardware`, `timebase`, `diagnostics`, etc.

---

## 4. Milestones

### P0 — Skeleton & IPC echo

- C++20 project scaffold with CMake.
- `SignalApp` that:
  - sets up logging,
  - initialises a stub `IpcServer` and `EngineHost`,
  - runs a simple main loop (or just start+shutdown for now).
- IPC layer:
  - `Envelope` struct mirroring Chorus envelope spec:
    - fields: `v`, `id`, `cid`, `ts`, `origin`, `target`, `domain`, `kind`, `name`, `priority`, `payload` (placeholder), `error`.
  - Decoder/encoder helpers for JSON (can be stubbed if networking isn't wired yet).
  - Domain router that:
    - logs any incoming messages,
    - dispatches to `engine` and `transport` domain handlers.

- Basic tests:
  - Envelope encode/decode roundtrip tests.
  - Domain router tests (dispatch by domain + name).

### P1 — Real IPC & Control Loop

- Implement actual IPC transport (most likely TCP) with newline-delimited JSON lines, matching Pulse.
- Add configuration for:
  - port,
  - host,
  - logging verbosity.
- Implement `engine` and `transport` domain command handling with in-memory state and events back to Pulse.

### P2 — Audio Engine Integration (Outline only)

- Introduce the real-time audio engine skeleton:
  - audio device abstraction,
  - processing graph skeleton,
  - timebase & clock.
- Wire `engine` and `transport` commands into the engine host.

### P3+ — Plugins, Hardware I/O, Advanced Domains

- Plugin hosting.
- Dynamic graph management.
- Multi-device routing.
- Networked / distributed Signal instances.

---

## 5. Testing Strategy

- **Unit tests:**
  - Envelope codec.
  - Domain router.
- **Integration tests:**
  - IPC server handling a small set of envelopes.
  - Engine host initialisation and shutdown.

---

## 6. Developer Workflow

- Use `AGENTS.md` as the primary instruction set for AI-assisted changes.
- Always:
  - Keep modules small and focused.
  - Maintain one major type per file (header/implementation pair).
  - Add or update tests when changing behaviour.
  - Keep the IPC layer consistent with Chorus envelope docs.

