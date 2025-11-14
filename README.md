<pre>
 ░▒▓███████▓▒░▒▓█▓▒░░▒▓██████▓▒░░▒▓███████▓▒░ ░▒▓██████▓▒░░▒▓█▓▒░
░▒▓█▓▒░      ░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░
░▒▓█▓▒░      ░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░
 ░▒▓██████▓▒░░▒▓█▓▒░▒▓█▓▒▒▓███▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓████████▓▒░▒▓█▓▒░
       ░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░
       ░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░
░▒▓███████▓▒░░▒▓█▓▒░░▒▓██████▓▒░░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓████████▓▒░

  L O O P H O L E - S I G N A L
</pre>

# Real-time audio engine server

Signal is the dedicated audio-engine process of the Loophole Digital Audio Workstation.
It is implemented in C++ using JUCE, and is responsible for real-time audio and MIDI
processing, plugin hosting, graph execution, and telemetry output.

Signal is designed to operate with strict real-time safety guarantees and runs in its
own process for isolation and performance.

---

## Contents

- [Purpose](#purpose)
- [Architecture Integration](#architecture-integration)
- [Responsibilities](#responsibilities)
- [Non-Responsibilities](#non-responsibilities)
- [Structure](#structure)
- [Development](#development)
- [Real-Time Safety](#real-time-safety)
- [Licence](#licence)

---

## Purpose

Signal provides the real-time backbone of Loophole.
Its responsibilities include:

- Real-time audio processing
- Plugin hosting (VST3, AU, CLAP)
- Audio and MIDI I/O
- Graph management and execution
- Sample-accurate parameter handling
- Emission of telemetry and analysis data
- Execution of engine commands sent by the UI and model layers

Signal does not manage project data, UI state, or higher-level editing logic.
Those responsibilities belong to Pulse and Aura, as defined in:

[`@chorus:/docs/architecture/01-overview.md`](https://github.com/infinite-loop-audio/loophole-chorus/blob/main/docs/architecture/01-overview.md)

---

## Architecture Integration

Signal participates in Loophole’s multi-process architecture:

- Aura sends engine-control commands to Signal.
- Pulse derives real-time-safe graph structures which Signal consumes.
- Signal provides telemetry and error reporting back to Aura.
- All interactions are defined by the IPC specifications in Chorus.

Relevant documents:

- Architecture Overview
  [`@chorus:/docs/architecture/01-overview.md`](https://github.com/infinite-loop-audio/loophole-chorus/blob/main/docs/architecture/01-overview.md)

- IPC Specifications
  [`@chorus:/docs/specs/`](https://github.com/infinite-loop-audio/loophole-chorus/blob/main/docs/specs/)

- Real-Time Safety Guidelines
  [`@chorus:/docs/specs/guidelines/realtime-safety.md`](https://github.com/infinite-loop-audio/loophole-chorus/blob/main/docs/specs/guidelines/realtime-safety.md)

Signal MUST conform to these specifications.

---

## Responsibilities

Signal is responsible for:

- Performing audio and MIDI processing within real-time deadlines
- Maintaining one or more plugin processing graphs
- Hosting third-party plugins safely and defensively
- Applying parameter changes with sample-accurate timing
- Managing low-latency device interactions
- Emitting telemetry for metering, timing and analysis
- Ensuring engine stability even under heavy UI or system load

Signal is the only component of Loophole that executes time-critical code.

---

## Non-Responsibilities

Signal intentionally does **not**:

- Store the project model
- Manage routing or track structures beyond RT graph execution
- Handle UI logic or plugin windowing
- Perform disk operations
- Make scheduling decisions outside the RT constraints
- Encode business logic for editing or arrangement

These are the responsibilities of Pulse and Aura.

---

## Structure

A typical layout for this repository will include:

```
src/
  engine/
  graph/
  plugins/
  devices/
  telemetry/
  ipc/
tests/
cmake/
resources/
```

This structure may evolve as the engine develops and as IPC and specification
requirements expand within Chorus.

---

## Development

Signal is built using CMake.

To configure and build:

```
cmake -B build
cmake --build build --config Release
```

Debug builds are also supported.

You may run Signal as a standalone process or allow Aura to launch and manage it.

The engine must be tested on macOS, Windows and Linux due to differences in device
APIs and plugin frameworks.

---

## Real-Time Safety

Signal MUST adhere to the real-time rules defined in Chorus:

[`@chorus:/docs/specs/guidelines/realtime-safety.md`](https://github.com/infinite-loop-audio/loophole-chorus/blob/main/docs/specs/guidelines/realtime-safety.md)

In particular, real-time code must not:

- Allocate or free memory
- Acquire locks or use blocking operations
- Perform I/O
- Use dynamic container resizing
- Execute unbounded computations

Signal should treat plugin code as untrusted and must be defensive at API boundaries.

---

## Licence

Signal is provided under the MIT Licence with the following additional clause:

**The Loophole name (including its components: Signal, Pulse, Aura and Chorus)
may not be used to promote or endorse any derived product without prior written
permission from the copyright holder.**

This clause applies to all repositories within the Loophole ecosystem.
