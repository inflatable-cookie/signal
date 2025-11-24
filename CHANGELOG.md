# Changelog

All notable changes to this project will be documented in this file.

This file follows a simplified [Keep a Changelog](https://keepachangelog.com/) style, with an ongoing **[Unreleased]** section and tagged entries.

Each entry in **[Unreleased]** must:

- Start with a UTC timestamp: `YYYY-MM-DD HH:MM:SS UTC`

- Include a tag in square brackets:
  - `[added]`   – new features or files
  - `[changed]` – behaviour changes, refactors, API tweaks
  - `[fixed]`   – bug fixes, stability improvements
  - `[removed]` – removed or deprecated features
  - `[docs]`    – documentation or spec changes
  - `[dev]`     – build, tests, tooling, CI

- End with a short, informative summary in British English.

Example entry:

`(2025-11-21 22:46:10 UTC) [changed] Normalised IPC event naming and removed response kind in favour of correlated events.`

## [Unreleased]

(2025-11-24 00:32:10 UTC) [added] Implemented audio engine runtime behaviour: integrated clip scheduling with audio callback, applied mixer gain/mute/solo in DSP path, implemented automation curve evaluation and application, added loop region wrapping in transport, and wired all systems together in real-time-safe audio processing pipeline.

(2025-11-23 23:28:41 UTC) [changed] Removed redundant DomainDispatcher and IpcRouter log messages, keeping only domain-specific logs to reduce log noise.

(2025-11-23 22:38:15 UTC) [fixed] Signal now emits engine.state events to newly connected clients, ensuring Aura receives notification of the current engine state when Pulse connects.

(2025-11-23 13:01:05 UTC) [changed] Hardened Signal skeleton with explicit concurrency model, proper engine lifecycle states, full transport domain handling, periodic diagnostics events, and graceful shutdown support.

(2025-11-23 03:00:00 UTC) [added] Initial Signal skeleton and Pulse ↔ Signal engine/transport bridge with minimal audio thread and IPC event support.

(2025-11-22 20:00:00 UTC) [added] Implemented Signal TCP IPC server handling JSON-line IpcEnvelopes with a central domain dispatcher stub.

(2025-01-27 00:00:00 UTC) [added] Initial C++20 project skeleton with CMake build system, IPC envelope structure, domain router, and test harness using Catch2.

