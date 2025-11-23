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

(2025-11-23 03:00:00 UTC) [added] Initial Signal skeleton and Pulse ↔ Signal engine/transport bridge with minimal audio thread and IPC event support.

(2025-11-22 20:00:00 UTC) [added] Implemented Signal TCP IPC server handling JSON-line IpcEnvelopes with a central domain dispatcher stub.

(2025-01-27 00:00:00 UTC) [added] Initial C++20 project skeleton with CMake build system, IPC envelope structure, domain router, and test harness using Catch2.

