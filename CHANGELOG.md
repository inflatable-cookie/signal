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

(2025-11-28 21:29:21 UTC) [added] Implemented engine self-test command: added offline render sanity check harness (EngineSelfTest) with 3 synthetic scenarios, IPC integration via EngineDomain, and diagnostics panel UI integration. Self-test runs short offline renders without touching live engine state and returns pass/fail summary.

(2025-11-28 21:07:47 UTC) [added] Added latency and tail handling stubs: node-level API (getLatencyInSamples, getTailInSamples, hasTailCurrently), graph-level aggregation methods, and EngineHost integration with atomic caching. All methods return zero (stub phase) but provide clean foundation for future latency compensation and tail-aware transport.

(2025-11-28 20:58:25 UTC) [changed] Enhanced graph snapshot channel metadata parsing to support separate input/output channel counts. GraphEngine now validates channel compatibility using explicit input/output channel counts from snapshot metadata, with improved error messages including node kind information.

(2025-11-28 20:47:41 UTC) [changed] Added explicit channel metadata validation in graph snapshot: Signal now validates `audio.channels` metadata from Pulse's graph snapshot, warns for missing metadata on required nodes, and validates channel compatibility at snapshot load time. GraphEngine prefers `audio.channels` over legacy `numAudioInputs`/`numAudioOutputs` fields.

(2025-11-28 20:38:53 UTC) [changed] Enhanced AudioBuffer::sumFrom() with channel-aware summing: now handles channel count mismatches with explicit upmix (duplicate last channel) and downmix (truncate extra channels) rules, supporting mono, stereo, and multi-channel layouts in the node-based mixer architecture.

(2025-11-28 20:40:00 UTC) [changed] Refactored MixerService to be fully channel-aware: finalMix() now handles mono, stereo, and multi-channel layouts correctly with panning only for stereo (2 channels) and gain applied uniformly to all channels, aligned with the unified node-based multi-channel model.

(2025-11-28 20:30:00 UTC) [changed] Enhanced DeviceNode multi-channel support: DeviceNode now handles channel count mismatches with explicit expansion (duplicate channels) and truncation (drop extra channels) logic, updated GraphEngine routing validation to allow DeviceNode channel mismatches with warnings, and improved logging for device channel configuration.

(2025-11-28 20:13:51 UTC) [changed] Extended DeviceNode to support multi-channel output devices: DeviceNode now queries active device channel count from EngineHost during prepare(), configures NodeAudioConfig to match device channels exactly, and GraphEngine validates device connections with strict channel matching.

(2025-11-28 20:06:11 UTC) [changed] Refined CLAP I/O negotiation to respect Pulse snapshot as source of truth: added _ioNegotiationOk flag to mark bypassed nodes, moved negotiation to prepare() after GraphEngine sets config, and implemented safe bypass behavior in process() when negotiation fails.

(2025-11-28 19:59:51 UTC) [changed] Implemented CLAP plugin audio I/O negotiation: plugins now query CLAP audio ports extension to determine actual I/O capabilities, negotiate with requested channel counts from Pulse snapshots, and update NodeAudioConfig accordingly. Added channel compatibility helper for routing validation.

(2025-11-28 19:53:58 UTC) [changed] Implemented strict multi-channel routing validation: connections must have matching channel counts, invalid connections are marked and excluded from routing, with comprehensive validation rules and error logging.

(2025-11-28 19:48:19 UTC) [changed] Unified channel configuration across all graph nodes: NodeAudioConfig is now the single source of truth, assigned from Pulse snapshot with node-type-specific defaults and connection validation.

(2025-11-28 19:41:06 UTC) [changed] Consolidated audio buffer types: AudioBuffer is canonical (deinterleaved), AudioBus is lightweight view (interleaved). Added efficient conversion utilities and eliminated redundant conversions throughout the engine.

(2025-11-28 19:38:04 UTC) [changed] Unified source and input node injection into a single Source/Input Pass that runs before node processing, eliminating duplication and clarifying render sequence responsibilities.

(2025-11-28 19:32:52 UTC) [changed] Refactored MeteringService to use lock-free atomic operations on audio thread with new submitSampleBlock() API, improving real-time safety while maintaining backward compatibility with Pulse IPC contract.

(2025-11-28 19:26:47 UTC) [changed] Unified MidiFxNode, InstrumentNode, and AudioFxNode into a single PluginNode class with PluginNodeKind enum, eliminating ~230 lines of duplicate code while preserving all existing behaviour and real-time safety guarantees.

(2025-11-28 18:15:00 UTC) [changed] TransportDomain now sends transport.positionUpdate event immediately when play/stop/seek commands are processed, ensuring Aura can sync its simulated play timer with Signal's actual playback start time.

(2025-11-28 17:59:22 UTC) [changed] Removed legacy IPC components (Router, Envelope, DomainHandler) and migrated all domains to pure IpcEnvelope handling via IpcDomainHandler interface. Simplified DomainDispatcher to registry-based forwarding only.

(2025-11-28 17:40:38 UTC) [changed] Refactored DomainDispatcher to registry pattern and moved all domain-specific logic into domain classes.

(2025-11-27 17:13:44 UTC) [fixed] Phase 12c: Fixed critical JSON parsing bug in MixerDomain - payload strings now correctly parsed before accessing JSON fields. Added MixerChannelNode gain application from MixerService updates.

(2025-11-27 16:50:00 UTC) [fixed] Phase 12b.5: Fixed AssetsDomain JSON parsing bug - registerAudioAsset payload was not being parsed from string, causing asset registration to fail and producing silence during playback.

(2025-11-27 16:45:00 UTC) [fixed] Phase 12b.5: Fixed graph engine processing streams when transport is stopped - streams are now only processed when ctx.isPlaying is true, preventing test tone from playing automatically.

(2025-11-27 16:32:01 UTC) [fixed] Phase 12b.5: Fixed critical bug where graph and schedule snapshot payloads were not being parsed from JSON strings, causing Signal to reject valid snapshots from Pulse. Also fixed playhead advancement to only occur during playback, not when stopped.

(2025-11-27 14:45:21 UTC) [dev] Phase 12b.5: Enhanced diagnostic logging for graph and schedule snapshot parsing, including raw JSON kind values, parsed NodeKind enum values, Device node counts, schedule array types/sizes, and detailed parsed stream/segment information to identify contract mismatches with Pulse.

(2025-11-27 14:20:52 UTC) [fixed] Phase 12b.4: Fixed GraphSnapshot JSON parsing with type-safe field access and comprehensive diagnostic logging, added diagnostic logging for schedule snapshot parsing to identify field name/type mismatches, and improved error handling for JSON type mismatches.

(2025-11-27 13:31:53 UTC) [dev] Phase 12b.3: Added comprehensive diagnostic logging and runtime probes throughout audio playback path, including debug checkpoints in EngineDomain and AssetsDomain, periodic render block logging with silence detection, runtime probes in AudioLaneNode and DeviceNode, diagnostic methods (hasGraph, hasSchedule, getActiveStreamCount), and headless offline render test for isolated audio processing verification.

(2025-11-26 23:36:33 UTC) [added] Phase 12a: Audio I/O & First Sound - Replaced placeholder MiniaudioBackend with real miniaudio integration, implemented device initialisation and enumeration, added runtime configuration (sample rate, buffer size, device name) to engine.state events, and wired device info flow from Signal to Pulse to Aura.

(2025-11-25 11:26:53 UTC) [added] Phase 9: Editing Engine - Extended AudioSegmentCompiled with fade metadata (fadeInSamples, fadeOutSamples, fade curves) and stretch metadata (StretchDescriptor with mode and ratio), added parsing in EngineDomain for fade/stretch from schedule JSON, and added TODO placeholders for future fade DSP and stretch algorithm implementation.

(2025-11-25 11:05:41 UTC) [added] Phase 8: Timebase & Transport Enhancements - Extended TransportState with sample-based loop regions, implemented loop wrapping in audio thread, added MusicalTimeInfo structure, and integrated transport/tempo info into NodeProcessContext for plugins.

(2025-11-25 09:09:00 UTC) [fixed] Implemented sigsetjmp/siglongjmp recovery mechanism to prevent Signal from crashing when loading problematic CLAP plugins - bus errors and segfaults during plugin loading now cause the plugin to be skipped rather than crashing Signal.

(2025-11-25 09:03:30 UTC) [dev] Added signal handlers for SIGBUS and SIGSEGV to identify which plugin causes crashes, with global state tracking to report the problematic plugin path in crash messages.

(2025-11-25 08:44:21 UTC) [dev] Added comprehensive logging throughout CLAP plugin loading process to help diagnose crashes, including detailed logging for each plugin file, path resolution, library loading steps, and error conditions with explicit flush() calls to ensure logs are visible before crashes.

(2025-11-25 08:37:03 UTC) [fixed] Deferred CLAP plugin scanning until after Signal's IPC server starts, preventing crashes from problematic plugins from blocking Signal startup and allowing Pulse to connect even if plugin scanning fails.

(2025-11-25 08:32:50 UTC) [fixed] Added error handling to prevent Signal from crashing when encountering problematic CLAP plugins during scanning, and added exception handling in main() and SignalApp initialization to provide better error reporting.

(2025-11-25 08:21:09 UTC) [fixed] Fixed CLAP plugin loading on macOS to correctly handle .clap bundles by resolving the actual library path from Contents/MacOS/ (handles files with or without extensions) and simplified ClapRegistry to delegate bundle resolution to ClapPluginLibrary.

(2025-11-25 07:27:09 UTC) [added] Phase 7: Recording & Live Input Integration - Added AudioInputNode and MidiInputNode for hardware input, RecordingSession for capture management, and real-time safe recording capture system.

(2025-11-25 00:10:40 UTC) [added] Implemented automation playback integration: AutomationData structures, block-time parameter application in renderBlock, routing to mixer nodes (gain/pan/send) and plugin nodes (CLAP parameters), with IPC handler for AutomationSnapshot from Pulse.

(2025-11-24 23:52:37 UTC) [added] Implemented real CLAP plugin loading and discovery with ClapPluginLibrary, ClapRegistry, and full CLAP API integration for plugin lifecycle, processing, parameters, and state.

(2025-11-24 23:34:41 UTC) [added] Implemented plugin hosting abstraction (PluginInstance, PluginHost) and CLAP adapter stub for MidiFxNode, InstrumentNode, and AudioFxNode. Added parameter change handling with lock-free queue and plugin state save/load hooks.

(2025-11-24 00:32:10 UTC) [added] Implemented audio engine runtime behaviour: integrated clip scheduling with audio callback, applied mixer gain/mute/solo in DSP path, implemented automation curve evaluation and application, added loop region wrapping in transport, and wired all systems together in real-time-safe audio processing pipeline.

(2025-11-23 23:28:41 UTC) [changed] Removed redundant DomainDispatcher and IpcRouter log messages, keeping only domain-specific logs to reduce log noise.

(2025-11-23 22:38:15 UTC) [fixed] Signal now emits engine.state events to newly connected clients, ensuring Aura receives notification of the current engine state when Pulse connects.

(2025-11-23 13:01:05 UTC) [changed] Hardened Signal skeleton with explicit concurrency model, proper engine lifecycle states, full transport domain handling, periodic diagnostics events, and graceful shutdown support.

(2025-11-23 03:00:00 UTC) [added] Initial Signal skeleton and Pulse ↔ Signal engine/transport bridge with minimal audio thread and IPC event support.

(2025-11-22 20:00:00 UTC) [added] Implemented Signal TCP IPC server handling JSON-line IpcEnvelopes with a central domain dispatcher stub.

(2025-01-27 00:00:00 UTC) [added] Initial C++20 project skeleton with CMake build system, IPC envelope structure, domain router, and test harness using Catch2.

