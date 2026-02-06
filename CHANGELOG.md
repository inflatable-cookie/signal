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
(2026-02-06 13:50:05 UTC) [added] Added a canonical `parameter` IPC domain handler in Signal with `requestDescriptors`, `requestValues`, and `setValue` command handling plus correlated `descriptorsSnapshot`/`valuesSnapshot`/`valueChanged` events.
(2026-02-06 13:50:05 UTC) [dev] Added binary-envelope-v2 coverage for `parameter.valuesSnapshot` and `parameter.valueChanged` and registered typed parameter payload codecs in the Signal build/runtime pipeline.
(2026-02-06 13:42:16 UTC) [changed] Extended VST3 state-chunk serialisation to persist canonical plugin parameter values and restored them through graph/plugin state reload paths.
(2026-02-06 13:42:16 UTC) [dev] Added Signal runtime-host test assertions that `bypass` parameter state round-trips through VST3 state chunks.
(2026-02-06 13:24:43 UTC) [changed] Added VST3 scaffold parameter value storage with clamping and bypass coercion, and extended plugin-hosting tests for parameter read/write behaviour.
(2026-02-06 13:23:32 UTC) [changed] Added canonical plugin parameter descriptor listing to the shared plugin instance API with CLAP and VST3 implementations plus runtime-host tests.
(2026-02-06 12:59:21 UTC) [fixed] Unblocked Signal builds by fixing MIDI polling lambda capture, correcting `MidiInputRouter` PIMPL wiring, and stabilising MIDI normaliser/plugin-hosting tests.
(2026-02-06 12:36:43 UTC) [changed] Wired plugin-state chunk restore on graph snapshot load and added GraphEngine state-chunk capture export for persisted plugin runtime state.
(2026-02-06 12:26:10 UTC) [changed] Extended the Phase 80 VST3 runtime scaffold with state-chunk roundtrip support and tests covering lifecycle, I/O negotiation, and restore semantics.
(2026-02-06 12:24:33 UTC) [changed] Added Phase 80.4 VST3 runtime node wiring with a host-created passthrough instance scaffold, shared I/O negotiation, and graph-path integration tests.
(2026-02-06 11:27:54 UTC) [changed] Added Phase 80.3 unified CLAP+VST3 plugin catalogue listing and per-format scan status counters with VST3 registry discovery tests.
(2026-02-06 11:24:25 UTC) [added] Added Phase 80 VST3 backend scaffolding with registry discovery, PluginHost factory routing, and optional SDK fetch/build wiring.
(2026-02-04 20:38:52 UTC) [added] Emitted control events from libremidi input using stable MIDI device identifiers.
(2026-02-04 19:19:23 UTC) [changed] Refined MIDI device ids with libremidi metadata and deterministic hashing.
(2026-02-04 19:06:28 UTC) [changed] Switched MIDI inventory enumeration to libremidi observer port metadata for stable identifiers.
(2026-02-04 18:57:16 UTC) [added] Added libremidi-based MIDI input enumeration for control device inventory.
(2026-02-04 18:40:29 UTC) [added] Emitted control device inventory snapshots on client connect and inventory changes.
(2025-12-24 19:28:51 UTC) [changed] Made `spatial.balance` a non-amplifying balance control and applied left/right group attenuation for common multichannel layouts (5.1/7.1/7.1.4), falling back safely for unknown layouts.
(2025-12-24 18:22:25 UTC) [added] Added `perChannelGain` spatial adapter support for FaderNodes and routed `spatial.channelGain.<index>` parameters.
(2025-12-24 16:46:07 UTC) [changed] Removed graph snapshot `mix.pan` initialisation and switched fader automation/control-plane to `parameterId="spatial.balance"`.
(2025-12-24 14:49:03 UTC) [changed] Removed remaining “channel-mix” terminology from Signal comments now that mix controls are node-owned.
(2025-12-24 12:21:43 UTC) [changed] Renamed graph snapshot node `channelMix` to `mix` (gain/pan only) across TLV decoding and graph-load application.
(2025-12-24 11:03:08 UTC) [removed] Removed `ChannelMixService` from the audio render path now that mute/gain are owned by graph nodes.
(2025-12-24 10:53:19 UTC) [removed] Removed the unused `channelMix` IPC domain handler now that mute is expressed via node parameters.
(2025-12-24 10:23:01 UTC) [changed] Removed `muted`/`soloed` decoding from the `engine.graphSnapshot` channel-mix TLV payload, consolidating mute on `node.setParameter`.
(2025-12-24 08:50:05 UTC) [changed] Stopped applying graph snapshot `mixer.muted` flags at load time; mute is now driven solely by the `node.setParameter` (`muted`) control-plane.
(2025-12-24 08:37:25 UTC) [added] Implemented `muted` handling for plugin nodes via `node.setParameter` and added a regression test ensuring muted plugin nodes output silence.
(2025-12-24 08:26:34 UTC) [changed] Applied mute via `node.setParameter` (`muted`) on FaderNodes and removed the `channelMix.updateChannel` binary decoder path.
(2025-12-24 07:00:29 UTC) [fixed] Added binary-envelope-v2 TLV encoding for `metering.update` and relaxed timestamp parsing to avoid spurious encode failures.
(2025-12-24 06:34:55 UTC) [fixed] Dropped unsupported outbound binary envelopes instead of closing the Pulse↔Signal control-plane connection, preventing Signal disconnect loops.
(2025-12-22 23:22:43 UTC) [removed] Removed kind=1 JSON frames and the JSON envelope codec from the Pulse↔Signal LPF1 control-plane (binary-envelope-v2 only).

(2025-12-22 21:31:49 UTC) [added] Added binary-envelope-v2 TLV decoding for `engine.heartbeat` and `engine.selfTest` commands over LPF1 framing.

(2025-12-22 21:24:29 UTC) [removed] Removed unused JSON-string TLV decoding helper now that runtime-push commands are fully typed.

(2025-12-22 21:19:02 UTC) [changed] Replaced JSON-string parsing for `automation.automationSnapshot` with typed TLV decoding (rebuilding the JSON object from TLV).

(2025-12-22 21:12:05 UTC) [changed] Replaced JSON-string parsing for `engine.playbackScheduleSnapshot` with typed TLV decoding (rebuilding the JSON object from TLV).

(2025-12-22 21:00:28 UTC) [changed] Replaced JSON-string parsing for `engine.graphSnapshot` with typed TLV decoding (rebuilding the JSON object from TLV).

(2025-12-22 20:46:24 UTC) [added] Added binary-envelope-v2 TLV decoding for `engine.start` and `engine.stop` commands over LPF1 framing.

(2025-12-22 20:41:40 UTC) [added] Added binary-envelope-v2 TLV decoding for core `transport` commands (play/stop/seek/loop) over LPF1 framing.

(2025-12-22 19:51:42 UTC) [changed] Removed legacy `hardware.listOutputDevices` and `hardware.setActiveOutputDevice` aliases in favour of `refreshOutputDevices` and `selectOutputDevice` only.

(2025-12-22 19:47:32 UTC) [changed] Normalised hardware responses to emit `hardware.state` and added TLV encoding support for binary-envelope-v2 framed events.

(2025-12-22 17:57:56 UTC) [added] Added an experimental framed-binary `binary-envelope-v2` decoder for the Pulse→Signal pilot (`assets.registerAudioAsset`), auto-detected via LPF1 magic.

(2025-12-22 18:07:25 UTC) [changed] Extended framed-binary control-plane IPC to accept kind=1 JSON envelope frames alongside kind=3 `binary-envelope-v2` frames.

(2025-12-22 18:46:09 UTC) [changed] Signal now emits JSON envelopes as framed kind=1 messages when the control-plane connection is in LPF1 framed mode.

(2025-12-22 18:53:28 UTC) [added] Signal can now emit `engine.state` as kind=3 `binary-envelope-v2` frames (TLV payload), falling back to kind=1 JSON for all other messages.

(2025-12-22 18:56:59 UTC) [added] Signal can now emit `transport.state` as kind=3 `binary-envelope-v2` frames (TLV payload), falling back to kind=1 JSON for all other messages.

(2025-12-22 19:00:31 UTC) [added] Signal can now emit `transport.positionUpdate` as kind=3 `binary-envelope-v2` frames (TLV payload), falling back to kind=1 JSON for all other messages.

(2025-12-22 19:03:56 UTC) [added] Signal can now emit `engine.selfTestResult` as kind=3 `binary-envelope-v2` frames (TLV payload), including object-list encoding for self-test scenarios.

(2025-12-22 19:06:53 UTC) [added] Signal can now decode kind=3 `binary-envelope-v2` frames for `engine.graphSnapshot` and `engine.playbackScheduleSnapshot` (TLV payload contains JSON string).

(2025-12-22 19:11:10 UTC) [added] Signal can now decode kind=3 `binary-envelope-v2` frames for `automation.automationSnapshot` (TLV payload contains JSON string).

(2025-12-22 19:15:21 UTC) [added] Signal can now decode kind=3 `binary-envelope-v2` frames for `channelMix.updateChannel` (TLV typed payload).

(2025-12-22 19:23:24 UTC) [changed] Removed legacy newline-delimited JSON IPC for the Pulse control-plane and now requires LPF1 framing (with kind=1 JSON frames where needed).

(2025-12-22 19:30:53 UTC) [added] Signal can now decode kind=3 `binary-envelope-v2` frames for `node.setParameter` (TLV typed payload).

(2025-12-22 19:35:41 UTC) [added] Signal can now decode kind=3 `binary-envelope-v2` frames for `hardware.refreshOutputDevices` and `hardware.selectOutputDevice` (TLV typed payloads).

(2025-12-15 21:49:21 UTC) [changed] Renamed the Signal `channelMix` IPC handler implementation from MixerDomain to ChannelMixDomain and removed remaining MixerDomain references.

(2025-12-15 21:30:23 UTC) [fixed] Removed undefined behaviour in test and runtime schedule/graph DTOs by default-initialising connection indices and audio segment metadata.

(2025-12-15 21:30:23 UTC) [changed] EngineHost now selects a specific HardwareAudioOutputNode (preferring the default device and an explicit output FaderNode upstream) and skips the redundant final gain stage when that output fader is present.

(2025-12-11 22:22:28 UTC) [changed] Stopped MixerDomain from writing FaderNode gain and pan so Fader parameters are owned exclusively by the node.setParameter IPC path while MixerService continues to manage consolidated mixer state.

(2025-12-11 19:00:00 UTC) [changed] Aligned Signal graph node kinds and mixer handling with the Fader-based GraphSnapshot contract by treating `kind: "fader"` and `fader-*` IDs as the primary mixer nodes instead of the legacy `mixer-channel` naming.

(2025-12-10 14:15:00 UTC) [dev] Tightened the existing send/receive runtime test to assert non-zero device output for the 440 Hz test tone and added a simple subgroup routing test that validates MixerChannel → bus → Device topology for Phase 7 routing scenarios.

(2025-12-10 14:03:00 UTC) [changed] Wired MixerChannelNode initial gain/pan/mute state to the new mixer metadata in GraphSnapshot so Signal’s runtime graph starts in sync with Pulse’s Channel model before mixer.updateChannel and automation updates arrive.

(2025-12-10 09:43:08 UTC) [dev] Added a minimal package.json with pnpm wrapper scripts for CMake build, run, and test workflows so Signal can be driven alongside Aura using the same package manager entrypoints.

(2025-12-09 12:26:04 UTC) [fixed] Aligned EngineHost automation handling and tests with the consolidated AutomationService, ensuring mixer/send parameters respect node IDs and block-time evaluation while keeping the idle fast-path semantics intact.
(2025-12-09 12:26:04 UTC) [fixed] Hardened GraphEngine phase 3 runtime tests by fixing stream injection, clearing node input/output buffers per block, and updating schedule-driven lane behaviour and routing validation to match the current engine design.
(2025-12-09 12:10:30 UTC) [dev] Fixed Signal offline playback path tests by preparing the graph after loading the test graph snapshot so EngineHost::renderBlock exercises the real AudioLane → Device graph with the stub test tone asset.

(2025-12-01 01:06:10 UTC) [docs] Documented canonical graph/render ingestion and dispatcher pattern in AGENTS and trimmed unused include from DomainDispatcher to keep the dispatch surface lean.
(2025-11-28 21:51:02 UTC) [changed] Unified all logging to use DEBUG_LEVEL system: converted all std::cout/cerr calls to unified logging macros, demoted noisy per-plugin logs from Info to Debug, and standardised area prefixes. Signal now matches Pulse's quiet logging profile at default DEBUG_LEVEL=4.

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
