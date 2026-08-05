# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/).
During v0.x, MINOR bumps may include breaking changes.
Signal's crates are not published to crates.io. Consumers pin a tag and
reference the crates by git, so a release is a tagged commit rather than a
registry upload.

## [Unreleased]

### Changed

- Froze `g10.040` Batch 40.2, the preview streaming brief. Ratio range
  `[0.25, 3.0]` with both ends derived: the maximum is Contract `046`'s overlap
  law at the `128`/`512` geometry, the minimum is bounded work at `2.36%` of a
  stereo `128`-frame callback. A `1 MiB` stereo ceiling at `MAX_BLOCK_FRAMES`
  against a measured-plus-computed `804.3 KiB`, derived after the design rather
  than before it. One ratio scheduler: the source-projection set survives and
  the output-side duplicate is deleted, because the projection already computes
  the source advance the streaming model needs — it was never wrong, nothing
  consumed it. Underrun must report a shortfall rather than return a block
  indistinguishable from a normal one, which is how the present defect stayed
  hidden for three roadmaps.
- Decided `g10.040` Batch 40.1: the RealtimePreview callback tier is reachable.
  Measuring the kernel first reframed the lane — stereo at ratio `1.0` uses
  `0.6%` of a `128`-frame callback budget and one-sixteenth speed uses `9.4%`,
  so CPU was never what stalled it across three roadmaps. The defect is that
  `process` consumes and produces the same frame count regardless of ratio,
  which diverges the analysis and synthesis cursors until the input-ring guard
  discards unanalysed source and still returns `Ok`. Bounded work additionally
  needs a frozen minimum ratio, since load scales as `1/ratio` while
  `sanitize_ratio` accepts any positive value.

## [0.1.0] - 2026-08-05

### Added

- `ResumableOfflineStretch` carries phase, detector, and overlap-add state across chunk boundaries, so a source rendered in any number of chunks is bit-identical to one rendered whole. The offline artifact path uses it for the default path without pitch shift.
- Promoted Signal's official demo task pack into native Effigy `[demos.*]` registry entries so the current browser, inspect, history, and live terminal surfaces can discover the repo's real demo cohort directly.
- Added a canonical `plugin` domain handler with `plugin.list` support that returns the current scanned CLAP/VST3 catalogue as correlated events.
- Added a canonical `parameter` IPC domain handler in Signal with `requestDescriptors`, `requestValues`, and `setValue` command handling plus correlated `descriptorsSnapshot`/`valuesSnapshot`/`valueChanged` events.
- Added Phase 80 VST3 backend scaffolding with registry discovery, PluginHost factory routing, and optional SDK fetch/build wiring.
- Emitted control events from libremidi input using stable MIDI device identifiers.
- Added libremidi-based MIDI input enumeration for control device inventory.
- Emitted control device inventory snapshots on client connect and inventory changes.
- Added `perChannelGain` spatial adapter support for FaderNodes and routed `spatial.channelGain.<index>` parameters.
- Implemented `muted` handling for plugin nodes via `node.setParameter` and added a regression test ensuring muted plugin nodes output silence.
- Added binary-envelope-v2 TLV decoding for `engine.heartbeat` and `engine.selfTest` commands over LPF1 framing.
- Added binary-envelope-v2 TLV decoding for `engine.start` and `engine.stop` commands over LPF1 framing.
- Added binary-envelope-v2 TLV decoding for core `transport` commands (play/stop/seek/loop) over LPF1 framing.
- Added an experimental framed-binary `binary-envelope-v2` decoder for the Pulse→Signal pilot (`assets.registerAudioAsset`), auto-detected via LPF1 magic.
- Signal can now emit `engine.state` as kind=3 `binary-envelope-v2` frames (TLV payload), falling back to kind=1 JSON for all other messages.
- Signal can now emit `transport.state` as kind=3 `binary-envelope-v2` frames (TLV payload), falling back to kind=1 JSON for all other messages.
- Signal can now emit `transport.positionUpdate` as kind=3 `binary-envelope-v2` frames (TLV payload), falling back to kind=1 JSON for all other messages.
- Signal can now emit `engine.selfTestResult` as kind=3 `binary-envelope-v2` frames (TLV payload), including object-list encoding for self-test scenarios.
- Signal can now decode kind=3 `binary-envelope-v2` frames for `engine.graphSnapshot` and `engine.playbackScheduleSnapshot` (TLV payload contains JSON string).
- Signal can now decode kind=3 `binary-envelope-v2` frames for `automation.automationSnapshot` (TLV payload contains JSON string).
- Signal can now decode kind=3 `binary-envelope-v2` frames for `channelMix.updateChannel` (TLV typed payload).
- Signal can now decode kind=3 `binary-envelope-v2` frames for `node.setParameter` (TLV typed payload).
- Signal can now decode kind=3 `binary-envelope-v2` frames for `hardware.refreshOutputDevices` and `hardware.selectOutputDevice` (TLV typed payloads).
- Implemented engine self-test command: added offline render sanity check harness (EngineSelfTest) with 3 synthetic scenarios, IPC integration via EngineDomain, and diagnostics panel UI integration. Self-test runs short offline renders without touching live engine state and returns pass/fail summary.
- Added latency and tail handling stubs: node-level API (getLatencyInSamples, getTailInSamples, hasTailCurrently), graph-level aggregation methods, and EngineHost integration with atomic caching. All methods return zero (stub phase) but provide clean foundation for future latency compensation and tail-aware transport.
- Phase 12a: Audio I/O & First Sound - Replaced placeholder MiniaudioBackend with real miniaudio integration, implemented device initialisation and enumeration, added runtime configuration (sample rate, buffer size, device name) to engine.state events, and wired device info flow from Signal to Pulse to Aura.
- Phase 9: Editing Engine - Extended AudioSegmentCompiled with fade metadata (fadeInSamples, fadeOutSamples, fade curves) and stretch metadata (StretchDescriptor with mode and ratio), added parsing in EngineDomain for fade/stretch from schedule JSON, and added TODO placeholders for future fade DSP and stretch algorithm implementation.
- Phase 8: Timebase & Transport Enhancements - Extended TransportState with sample-based loop regions, implemented loop wrapping in audio thread, added MusicalTimeInfo structure, and integrated transport/tempo info into NodeProcessContext for plugins.
- Phase 7: Recording & Live Input Integration - Added AudioInputNode and MidiInputNode for hardware input, RecordingSession for capture management, and real-time safe recording capture system.
- Implemented automation playback integration: AutomationData structures, block-time parameter application in renderBlock, routing to mixer nodes (gain/pan/send) and plugin nodes (CLAP parameters), with IPC handler for AutomationSnapshot from Pulse.
- Implemented real CLAP plugin loading and discovery with ClapPluginLibrary, ClapRegistry, and full CLAP API integration for plugin lifecycle, processing, parameters, and state.
- Implemented plugin hosting abstraction (PluginInstance, PluginHost) and CLAP adapter stub for MidiFxNode, InstrumentNode, and AudioFxNode. Added parameter change handling with lock-free queue and plugin state save/load hooks.
- Implemented audio engine runtime behaviour: integrated clip scheduling with audio callback, applied mixer gain/mute/solo in DSP path, implemented automation curve evaluation and application, added loop region wrapping in transport, and wired all systems together in real-time-safe audio processing pipeline.
- Initial Signal skeleton and Pulse ↔ Signal engine/transport bridge with minimal audio thread and IPC event support.
- Implemented Signal TCP IPC server handling JSON-line IpcEnvelopes with a central domain dispatcher stub.
- Initial C++20 project skeleton with CMake build system, IPC envelope structure, domain router, and test harness using Catch2.

### Changed

- Admitted the resumable offline stretch renderer on the default offline path
  with no pitch shift, closing `g10.039`. It carries phase-vocoder state across
  chunk and dynamic-ratio boundaries instead of restarting at each join, and
  chunked output is now bit-identical across chunk policies (correlation
  `1.000000`, against `0.389976` for the per-chunk renderer it replaces).
  Concealed listening found no significant difference between the two, which
  admits it as parity rather than as a demonstrated improvement. Selector paths
  and pitch composition still take the legacy per-chunk path, so both seam
  smoothers remain.
- Cleared all 14 clippy warnings and tightened both lint gates to `-D warnings`,
  so new lint debt blocks a release instead of accumulating behind a passing
  signal.
- Grouped the RealtimePreview dynamic source projection builder's eight
  ratio parameters into `DynamicSourceProjectionRatios`; two call sites had been
  passing thirteen positional arguments.
- `TimeStretcher` and the whole-buffer stretch entry points are now fallible and refuse renders above a `268435456`-sample ceiling. A `1.0e6` ratio previously attempted a `4096000000`-sample allocation.
- Stretch cache identity advances to `signal-stretch-cache-v3`. Render geometry, chunk policy, and a crate-owned behaviour version join the key, and tier and offline path are stable tokens rather than `Debug` output. Every `v2` artifact is invalid: it was keyed without inputs that change rendered audio.
- Creative stretch renders are declared uncacheable in Contract `085` rather than left undeclared.
- `signal-dsp-stretch` consolidation: one promotion-gate owner instead of three, one shared spectral surface instead of two copies, one transient-smear entry point instead of four, the RealtimePreview tier moved out of `lib.rs`, and caller-owned FFT scratch cutting a four-second render from `789` to `31` allocations.
- Renamed `signal-hardware-output-cpal` to `signal-hardware-cpal`: the crate has carried input streams (`CpalInputBackend`, `enumerate_input_devices`) alongside output since recording v1, so the output-only name was historical.
- [dev] g10.009 workspace consolidation: `[workspace.dependencies]` for all shared internal/external deps with unified versions, `[workspace.lints]` wired into every crate, rustfmt/rust-toolchain pins, clippy taken from 44 warnings to zero, GitHub Actions CI (build/test/fmt/clippy), and the README/system inventory rewritten to the post-g10 crate set with the production audio path documented first.
- [dev] Widened runtime-owned plugin discovery receipts with format-coverage and backend-neutral capability aggregates for the opening `g05` backend-breadth tranche.
- [docs] Documented the widened `g05.001` plugin discovery receipt boundary and moved the roadmap queue to conformance proof.
- [dev] Added a combined `g04` closeout description mode and repo-owned acceptance task for conformance, release baseline, and post-generation queue handoff.
- [docs] Closed `g04` and recorded the explicit post-`g04` consumer/release/backend breadth backlog candidate.
- [dev] Added a host-free release-boundary description mode and repo-owned acceptance task for Signal's first packaging baseline.
- [docs] Documented the runnable consumer conformance and release-packaging baseline for the stabilised runtime/export boundary.
- [dev] Added native Effigy docs QA tasks and declarative docs-policy checks for Signal's Northstar spine.
- [docs] Updated the README and AGENTS guide to teach repo-root Effigy usage without redundant `--repo .` defaults.
- Registered typed binary-envelope-v2 codecs for `plugin.list` command/event payloads in the Signal codec registry and build graph.
- Parameter domain now enforces canonical `scope.pluginInstanceId` matching against the active plugin node instance and returns scope-invalid errors when mismatched.
- Graph snapshot parsing and PluginNode runtime identity now carry `pluginInstanceId`, and parameter responses emit the canonical scope identity from runtime state.
- [dev] Extended plugin graph-hosting coverage to assert configured `pluginInstanceId` is retained on runtime plugin nodes.
- Emitted `diagnostics.error` (`engine.pluginUnavailableOnRestore`) after graph snapshot loads with unavailable plugin nodes so Pulse/Aura can surface degraded restore state.
- [dev] Added graph-load regression coverage for tracking unavailable plugin nodes when plugin instantiation fails.
- Made automation snapshot ingestion deterministic for equal-time events by sorting on time samples, node id, then parameter id in `AutomationDomain`.
- [dev] Expanded automation sorting tests to assert stable tie-break ordering for equal-time events.
- [dev] Added binary-envelope-v2 coverage for `parameter.valuesSnapshot` and `parameter.valueChanged` and registered typed parameter payload codecs in the Signal build/runtime pipeline.
- Extended VST3 state-chunk serialisation to persist canonical plugin parameter values and restored them through graph/plugin state reload paths.
- [dev] Added Signal runtime-host test assertions that `bypass` parameter state round-trips through VST3 state chunks.
- Added VST3 scaffold parameter value storage with clamping and bypass coercion, and extended plugin-hosting tests for parameter read/write behaviour.
- Added canonical plugin parameter descriptor listing to the shared plugin instance API with CLAP and VST3 implementations plus runtime-host tests.
- Wired plugin-state chunk restore on graph snapshot load and added GraphEngine state-chunk capture export for persisted plugin runtime state.
- Extended the Phase 80 VST3 runtime scaffold with state-chunk roundtrip support and tests covering lifecycle, I/O negotiation, and restore semantics.
- Added Phase 80.4 VST3 runtime node wiring with a host-created passthrough instance scaffold, shared I/O negotiation, and graph-path integration tests.
- Added Phase 80.3 unified CLAP+VST3 plugin catalogue listing and per-format scan status counters with VST3 registry discovery tests.
- Refined MIDI device ids with libremidi metadata and deterministic hashing.
- Switched MIDI inventory enumeration to libremidi observer port metadata for stable identifiers.
- Made `spatial.balance` a non-amplifying balance control and applied left/right group attenuation for common multichannel layouts (5.1/7.1/7.1.4), falling back safely for unknown layouts.
- Removed graph snapshot `mix.pan` initialisation and switched fader automation/control-plane to `parameterId="spatial.balance"`.
- Removed remaining “channel-mix” terminology from Signal comments now that mix controls are node-owned.
- Renamed graph snapshot node `channelMix` to `mix` (gain/pan only) across TLV decoding and graph-load application.
- Removed `muted`/`soloed` decoding from the `engine.graphSnapshot` channel-mix TLV payload, consolidating mute on `node.setParameter`.
- Stopped applying graph snapshot `mixer.muted` flags at load time; mute is now driven solely by the `node.setParameter` (`muted`) control-plane.
- Applied mute via `node.setParameter` (`muted`) on FaderNodes and removed the `channelMix.updateChannel` binary decoder path.
- Replaced JSON-string parsing for `automation.automationSnapshot` with typed TLV decoding (rebuilding the JSON object from TLV).
- Replaced JSON-string parsing for `engine.playbackScheduleSnapshot` with typed TLV decoding (rebuilding the JSON object from TLV).
- Replaced JSON-string parsing for `engine.graphSnapshot` with typed TLV decoding (rebuilding the JSON object from TLV).
- Removed legacy `hardware.listOutputDevices` and `hardware.setActiveOutputDevice` aliases in favour of `refreshOutputDevices` and `selectOutputDevice` only.
- Normalised hardware responses to emit `hardware.state` and added TLV encoding support for binary-envelope-v2 framed events.
- Extended framed-binary control-plane IPC to accept kind=1 JSON envelope frames alongside kind=3 `binary-envelope-v2` frames.
- Signal now emits JSON envelopes as framed kind=1 messages when the control-plane connection is in LPF1 framed mode.
- Removed legacy newline-delimited JSON IPC for the Pulse control-plane and now requires LPF1 framing (with kind=1 JSON frames where needed).
- Renamed the Signal `channelMix` IPC handler implementation from MixerDomain to ChannelMixDomain and removed remaining MixerDomain references.
- EngineHost now selects a specific HardwareAudioOutputNode (preferring the default device and an explicit output FaderNode upstream) and skips the redundant final gain stage when that output fader is present.
- Stopped MixerDomain from writing FaderNode gain and pan so Fader parameters are owned exclusively by the node.setParameter IPC path while MixerService continues to manage consolidated mixer state.
- Aligned Signal graph node kinds and mixer handling with the Fader-based GraphSnapshot contract by treating `kind: "fader"` and `fader-*` IDs as the primary mixer nodes instead of the legacy `mixer-channel` naming.
- [dev] Tightened the existing send/receive runtime test to assert non-zero device output for the 440 Hz test tone and added a simple subgroup routing test that validates MixerChannel → bus → Device topology for Phase 7 routing scenarios.
- Wired MixerChannelNode initial gain/pan/mute state to the new mixer metadata in GraphSnapshot so Signal’s runtime graph starts in sync with Pulse’s Channel model before mixer.updateChannel and automation updates arrive.
- [dev] Added a minimal package.json with pnpm wrapper scripts for CMake build, run, and test workflows so Signal can be driven alongside Aura using the same package manager entrypoints.
- [dev] Fixed Signal offline playback path tests by preparing the graph after loading the test graph snapshot so EngineHost::renderBlock exercises the real AudioLane → Device graph with the stub test tone asset.
- [docs] Documented canonical graph/render ingestion and dispatcher pattern in AGENTS and trimmed unused include from DomainDispatcher to keep the dispatch surface lean.
- Unified all logging to use DEBUG_LEVEL system: converted all std::cout/cerr calls to unified logging macros, demoted noisy per-plugin logs from Info to Debug, and standardised area prefixes. Signal now matches Pulse's quiet logging profile at default DEBUG_LEVEL=4.
- Enhanced graph snapshot channel metadata parsing to support separate input/output channel counts. GraphEngine now validates channel compatibility using explicit input/output channel counts from snapshot metadata, with improved error messages including node kind information.
- Added explicit channel metadata validation in graph snapshot: Signal now validates `audio.channels` metadata from Pulse's graph snapshot, warns for missing metadata on required nodes, and validates channel compatibility at snapshot load time. GraphEngine prefers `audio.channels` over legacy `numAudioInputs`/`numAudioOutputs` fields.
- Enhanced AudioBuffer::sumFrom() with channel-aware summing: now handles channel count mismatches with explicit upmix (duplicate last channel) and downmix (truncate extra channels) rules, supporting mono, stereo, and multi-channel layouts in the node-based mixer architecture.
- Refactored MixerService to be fully channel-aware: finalMix() now handles mono, stereo, and multi-channel layouts correctly with panning only for stereo (2 channels) and gain applied uniformly to all channels, aligned with the unified node-based multi-channel model.
- Enhanced DeviceNode multi-channel support: DeviceNode now handles channel count mismatches with explicit expansion (duplicate channels) and truncation (drop extra channels) logic, updated GraphEngine routing validation to allow DeviceNode channel mismatches with warnings, and improved logging for device channel configuration.
- Extended DeviceNode to support multi-channel output devices: DeviceNode now queries active device channel count from EngineHost during prepare(), configures NodeAudioConfig to match device channels exactly, and GraphEngine validates device connections with strict channel matching.
- Refined CLAP I/O negotiation to respect Pulse snapshot as source of truth: added _ioNegotiationOk flag to mark bypassed nodes, moved negotiation to prepare() after GraphEngine sets config, and implemented safe bypass behavior in process() when negotiation fails.
- Implemented CLAP plugin audio I/O negotiation: plugins now query CLAP audio ports extension to determine actual I/O capabilities, negotiate with requested channel counts from Pulse snapshots, and update NodeAudioConfig accordingly. Added channel compatibility helper for routing validation.
- Implemented strict multi-channel routing validation: connections must have matching channel counts, invalid connections are marked and excluded from routing, with comprehensive validation rules and error logging.
- Unified channel configuration across all graph nodes: NodeAudioConfig is now the single source of truth, assigned from Pulse snapshot with node-type-specific defaults and connection validation.
- Consolidated audio buffer types: AudioBuffer is canonical (deinterleaved), AudioBus is lightweight view (interleaved). Added efficient conversion utilities and eliminated redundant conversions throughout the engine.
- Unified source and input node injection into a single Source/Input Pass that runs before node processing, eliminating duplication and clarifying render sequence responsibilities.
- Refactored MeteringService to use lock-free atomic operations on audio thread with new submitSampleBlock() API, improving real-time safety while maintaining backward compatibility with Pulse IPC contract.
- Unified MidiFxNode, InstrumentNode, and AudioFxNode into a single PluginNode class with PluginNodeKind enum, eliminating ~230 lines of duplicate code while preserving all existing behaviour and real-time safety guarantees.
- TransportDomain now sends transport.positionUpdate event immediately when play/stop/seek commands are processed, ensuring Aura can sync its simulated play timer with Signal's actual playback start time.
- Removed legacy IPC components (Router, Envelope, DomainHandler) and migrated all domains to pure IpcEnvelope handling via IpcDomainHandler interface. Simplified DomainDispatcher to registry-based forwarding only.
- Refactored DomainDispatcher to registry pattern and moved all domain-specific logic into domain classes.
- [dev] Phase 12b.5: Enhanced diagnostic logging for graph and schedule snapshot parsing, including raw JSON kind values, parsed NodeKind enum values, Device node counts, schedule array types/sizes, and detailed parsed stream/segment information to identify contract mismatches with Pulse.
- [dev] Phase 12b.3: Added comprehensive diagnostic logging and runtime probes throughout audio playback path, including debug checkpoints in EngineDomain and AssetsDomain, periodic render block logging with silence detection, runtime probes in AudioLaneNode and DeviceNode, diagnostic methods (hasGraph, hasSchedule, getActiveStreamCount), and headless offline render test for isolated audio processing verification.
- [dev] Added signal handlers for SIGBUS and SIGSEGV to identify which plugin causes crashes, with global state tracking to report the problematic plugin path in crash messages.
- [dev] Added comprehensive logging throughout CLAP plugin loading process to help diagnose crashes, including detailed logging for each plugin file, path resolution, library loading steps, and error conditions with explicit flush() calls to ensure logs are visible before crashes.
- Removed redundant DomainDispatcher and IpcRouter log messages, keeping only domain-specific logs to reduce log noise.
- Hardened Signal skeleton with explicit concurrency model, proper engine lifecycle states, full transport domain handling, periodic diagnostics events, and graceful shutdown support.

### Fixed

- Repaired CI, which had failed on every push since it was added. The toolchain
  step passed `--component rustfmt clippy`, so rustup read `clippy` as a
  toolchain name and exited before any build ran. Components are now declared in
  `rust-toolchain.toml`, and the workflow mirrors the cargo-based release gates
  so a green CI and a passing gate set mean the same thing.
- Pinned the toolchain to `1.97.1`. `channel = "stable"` floated, so the release
  gates ran on a stale local `1.96.0` while CI installed `1.97.1` — "the gates
  pass" and "CI passes" were claims about different compilers, and three clippy
  lints only the newer one knows went unseen.
- Removed three redundant `detail: _` patterns in `signal-runtime` transport
  fault matching, each sitting beside a `..` that already covered it.
- Moved seven wall-clock tests into an opt-in soak lane gated on
  `SIGNAL_SOAK_TESTS=1` and run by `effigy test:soak` single-threaded. Each
  sleeps for a fixed span and then asserts a minimum callback count, or asserts
  zero xruns — claims about host speed, not correctness, which no shared runner
  can honour. Findings `A20`, `A21` and `A22` were all this mechanism. The lane
  is a release gate, so the claim is still required before a tag.
- Closed finding `A22`. `signal-plugin-sandbox` `tests/plugin_hosting.rs` failed
  `2`, `5`, `6` and `7` of `12` under concurrent cargo activity; it was the soak
  tests holding callback threads hot that starved its timing budgets. With them
  gated, three consecutive workspace runs and ten consecutive runs of the binary
  are clean.
- Closed finding `A21` by removing an assertion that could not discriminate.
  `fake_clocked_soak` asserted the xrun counter grew by at most one after the
  injected starvation ended. Measured over a 1500ms window on an idle machine,
  the recovered phase accrues 2 to 8 xruns per ~281 callbacks, while the
  injected starvation rate over the same window is ~8.8 — the noise floor and
  the signal are the same magnitude, so it passed by luck. Removed rather than
  loosened; the starvation and playback-advancement claims still stand.
- Closed finding `A19`, a use-after-unmap in the shm round-trip test that had
  been carried since `g10.038` with no mechanism. It presented as an intermittent
  assertion failure *and* an intermittent `SIGSEGV`, and it was both from one
  cause: the retry loop was bounded by 200 iterations of the client thread,
  which is an assumption about host contention rather than a bound on the server
  thread being scheduled. When it lost, the panic unwound and dropped the shared
  region while the server thread was still dereferencing a raw pointer into it,
  killing the binary and taking the assertion message with it. The loop is now
  deadline-bounded and the join moved ahead of every assertion.

  Two further defects sat underneath, found once the segfault stopped hiding
  them. The fake child served exactly one request and exited, while every client
  retry issues a new request sequence — so answering request `N` after the
  client moved to `N+1` left a stale response and no possibility of another; it
  now serves until told to stop. And the client's wait budget is half a block,
  `333us`, so replacing the spin with a `1ms` server sleep polled three times
  slower than the window it had to answer within; the server is back to
  `yield_now`.

  None of that was why CI failed. `PLUGIN_PROCESS_CONSECUTIVE_TIMEOUT_LIMIT` is
  `3`: after three consecutive misses the processor clears `alive` and every
  later `process` returns false immediately, so a retry loop against a retired
  epoch is futile however long it runs. The three attempts have to land inside
  `min(1ms, half a block)`, which a contended three-core runner can miss three
  times in a row. The test now re-attaches on retirement and reports the epoch
  count.
- Removed the wall-clock throughput dependency from seven more tests in
  `signal-hardware`, missed by the first sweep because they live in `#[cfg(test)]`
  modules inside `src/` rather than under `tests/`. The two capture tests CI
  failed on required 50% of real-time; the two fake-backend cadence tests
  required 53% and were next. Cadence and allocation tests now poll for ten
  blocks with a deadline instead of sleeping a fixed span, so a slow host waits
  longer rather than failing; the capture tests keep their content assertions
  (tone phase at the skip point, RMS, zero-crossing rate) and drop their floors
  to a liveness minimum.
- Took `capture_callback_path_allocates_nothing` back out of the soak lane.
  Allocation-freedom on the capture callback holds at any speed and is worth
  checking on every run; only its block count was load-dependent.
- Serialised the eleven sandbox tests that spawn a child process. Each child
  runs a hot-spinning audio thread, and running twelve of them in parallel meant
  twelve spinning children plus twelve spinning parents — which fits on
  eighteen cores and does not on a CI runner's three, where children could not
  get enough CPU to answer inside their budget. It failed three tests, including
  one that surfaced as wrong audio rather than a timeout, because a missed
  response bypasses and leaves the scratch untouched. No timing budget was
  changed to fix it.
- Raised the sandbox child's first-response deadline from `5s` to `60s` and
  named it. It guards "did the child ever answer", and the first request waits
  on a real process spawn plus a plugin `dlopen`, so a `5s` bound measured the
  CI runner rather than the bridge. The `<20ms` bypass budgets are untouched:
  those assert that a dead child cannot block the audio thread, which is the
  product contract rather than scaffolding.
- Throttled the heavy release gates so the machine stays usable while they run:
  `nice -n 5`, two cores left free for builds and four for the test run.
  Unthrottled they saturated every core for minutes at a time. CI is left
  unthrottled, since a runner has nothing else to do.
- Narrowed CI triggers to `workflow_dispatch`, pull requests, and `v*` tags. It
  ran on every push to `main`, building the workspace twice under clippy plus a
  full test run on `macos-latest`, which bills at ten times the Linux rate.
- Marked `gui_open_embedded` `unsafe` on the VST3, AU, and CLAP host adapters.
  Each takes a caller-supplied raw parent-window pointer and hands it to FFI
  that attaches a view to it, so an invalid handle is undefined behaviour; the
  signature now says so and carries a documented safety contract.
- Time-stretch overlap coverage: the analysis hop now adapts so `analysis_hop * ratio` stays within `0.75 * window_size`. Ratios above `4.0` previously lost overlap-add coverage entirely, zeroing `183` of `547` interior blocks at ratio `6.0`; ratios through `3.0` are byte-identical to before.
- Dynamic-ratio curves sampled finer than one analysis window no longer degrade to varispeed. Short spans coalesce and render at their mean ratio with output length preserved exactly; a dense curve previously rendered a `440 Hz` source at `220 Hz`.
- Mono dynamic-ratio renders now receive the same segment-seam treatment as linked stereo, taking seam click from `-28.94` to `-180.62 dBFS`.
- Three test-integrity defects: two process-global allocation counters that attributed other threads work to the measuring test, and a `cfg(test)`-only path whose owner asserted behaviour production never serves.
- g10 production-path corrections: render plane declick (transport edge ramps, gain smoothing across plan swaps, clip-edge micro-fades, loop wrap), cpal output streams with real config negotiation and device enumeration (no `unsafe impl Send`), BS.1770-4 true peak via 4x polyphase FIR, LRA relative gate, complete-block loudness gating, polyphase windowed-sinc clip resampling on the audio thread (44.1k→48k SNR > 60 dB), ExponentialRamp/DelayLine fixes.
- Relaxed `plugin.list` command decode to accept the TLV payload shape emitted by Pulse, avoiding false trailing-bytes decode failures.
- Unblocked Signal builds by fixing MIDI polling lambda capture, correcting `MidiInputRouter` PIMPL wiring, and stabilising MIDI normaliser/plugin-hosting tests.
- Added binary-envelope-v2 TLV encoding for `metering.update` and relaxed timestamp parsing to avoid spurious encode failures.
- Dropped unsupported outbound binary envelopes instead of closing the Pulse↔Signal control-plane connection, preventing Signal disconnect loops.
- Removed undefined behaviour in test and runtime schedule/graph DTOs by default-initialising connection indices and audio segment metadata.
- Aligned EngineHost automation handling and tests with the consolidated AutomationService, ensuring mixer/send parameters respect node IDs and block-time evaluation while keeping the idle fast-path semantics intact.
- Hardened GraphEngine phase 3 runtime tests by fixing stream injection, clearing node input/output buffers per block, and updating schedule-driven lane behaviour and routing validation to match the current engine design.
- Phase 12c: Fixed critical JSON parsing bug in MixerDomain - payload strings now correctly parsed before accessing JSON fields. Added MixerChannelNode gain application from MixerService updates.
- Phase 12b.5: Fixed AssetsDomain JSON parsing bug - registerAudioAsset payload was not being parsed from string, causing asset registration to fail and producing silence during playback.
- Phase 12b.5: Fixed graph engine processing streams when transport is stopped - streams are now only processed when ctx.isPlaying is true, preventing test tone from playing automatically.
- Phase 12b.5: Fixed critical bug where graph and schedule snapshot payloads were not being parsed from JSON strings, causing Signal to reject valid snapshots from Pulse. Also fixed playhead advancement to only occur during playback, not when stopped.
- Phase 12b.4: Fixed GraphSnapshot JSON parsing with type-safe field access and comprehensive diagnostic logging, added diagnostic logging for schedule snapshot parsing to identify field name/type mismatches, and improved error handling for JSON type mismatches.
- Implemented sigsetjmp/siglongjmp recovery mechanism to prevent Signal from crashing when loading problematic CLAP plugins - bus errors and segfaults during plugin loading now cause the plugin to be skipped rather than crashing Signal.
- Deferred CLAP plugin scanning until after Signal's IPC server starts, preventing crashes from problematic plugins from blocking Signal startup and allowing Pulse to connect even if plugin scanning fails.
- Added error handling to prevent Signal from crashing when encountering problematic CLAP plugins during scanning, and added exception handling in main() and SignalApp initialization to provide better error reporting.
- Fixed CLAP plugin loading on macOS to correctly handle .clap bundles by resolving the actual library path from Contents/MacOS/ (handles files with or without extensions) and simplified ClapRegistry to delegate bundle resolution to ClapPluginLibrary.
- Signal now emits engine.state events to newly connected clients, ensuring Aura receives notification of the current engine state when Pulse connects.

### Removed

- g10.020 runtime endgame: shrank `signal-runtime` + `signal-host-local` to a thin control library (~52k → ~15k LoC of src). Deleted the engine-block simulation path, the anticipative prework scheduler (policy vocabulary preserved in `docs/architecture/prework-scheduler-design-note.md`), transport-session concurrency, deferred-service receipt stubs, metering/scheduler/timeline/automation narration snapshots, plugin recall/ARA/pin-matrix/spatial carve-outs, and the preview-transform/transform-artifact/stretch/marker stack with the clip-render simulation. `signal-graph` reduced to the plan model (execution engine deleted). Host-local boot no longer pumps simulated engine blocks; the reported stream state means a negotiated output stream. Pulse's consumed surface is unchanged (pulse builds and passes untouched).
- g10 demolition programme (packets 002-008): deleted ~98k LoC of simulated and narration-only code — `signal-supervisor-tools`, `signal-host-server`, `signal-hardware-coreaudio`, `signal-plugin-library`/`-store` crates removed; `signal-runtime` stripped of simulated posture domains and its narration layer (~29.4k); rhythm continuity taxonomy and embed model-registry fiction removed (~11.7k); plugin domain pruned to real discovery foundations with sandbox broker over verified shm leases (~20.8k); discovery roots now explicit configuration defaulting empty.
- Removed `ChannelMixService` from the audio render path now that mute/gain are owned by graph nodes.
- Removed the unused `channelMix` IPC domain handler now that mute is expressed via node parameters.
- Removed kind=1 JSON frames and the JSON envelope codec from the Pulse↔Signal LPF1 control-plane (binary-envelope-v2 only).
- Removed unused JSON-string TLV decoding helper now that runtime-push commands are fully typed.
