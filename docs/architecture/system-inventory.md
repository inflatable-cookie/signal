# System Inventory

Status: active
Owner: core-product
Updated: 2026-08-17
Vision refs: `docs/vision/001-signal-vision.md`

## Purpose

Record the execution-relevant Signal surface so roadmap and contract work can
sequence against explicit crate, boundary, and proof ownership instead of
implicit repo context.

This inventory reflects the post-`g10` workspace: the demolition packets
(g10.002-008) removed `signal-supervisor-tools`, `signal-host-server`,
`signal-hardware-coreaudio`, and `signal-plugin-library`/`-store`, and pruned
runtime, analysis, and plugin crates to their real surfaces. The pre-g10
inventory is in git history.

## Workspace Scope

Signal's active implementation surface is the Rust workspace under `crates/`
(28 crates).

## Layer Inventory

### Production audio path

- `signal-render-plane`
  - alloc-free realtime executor: compiled preallocated plans, declick
    envelopes (transport edges, gain smoothing, clip micro-fades), polyphase
    clip resampling, loop wrap
- `signal-hardware`
  - output stream contract: stream specs, negotiation types, device model
- `signal-hardware-cpal`
  - cpal-backed negotiated output streams and real device enumeration;
    thread-owned streams, zero unsafe; smoke tests self-skip without a device
- `signal-hardware-coremidi`
  - CoreMIDI-backed hardware MIDI input for macOS (handwritten FFI, no
    binding crate)

### Foundation and DSP substrate

- `signal-primitives`
  - core sample, frame, transport, and channel-layout types
- `signal-dsp`
  - DSP kernels: ramps, smoothing, delay, and `PolyphaseInterpolationTable`
    (the RT-path interpolator used by the render plane)
- `signal-dsp-spectral`
  - FFT/STFT windows and spectral transforms
- `signal-dsp-resample`
  - deterministic offline/streaming mono resampler for analysis input prep;
    not the realtime path
- `signal-dsp-stretch`
  - frozen Signal-owned offline and preview time-stretch baselines, creative
    renderers, cache identity, promotion receipts, and callback-state proof;
    transparent successor admission is closed under
    `g10.030`; `g10.031` and Contract `085` publicly admit exact fixed `4x`,
    `8x`, and `16x` neutral `Dream` through an offline whole-buffer API;
    automatic routing, creative cache/artifacts, dynamic ratio, and product
    integration remain absent
- `signal-dsp-stretch-evidence`
  - comparator, corpus-selection, behavioural-probe, and blind-listening
    command tools; enables the stretch crate's opt-in `evidence` API without
    entering production render or artifact-planning dependency graphs

### Analysis substrate

- `signal-analysis`
  - shared analysis traits, result types, and input preparation; the
    corpus/acceptance harness is test infrastructure behind the
    `test-support` feature
- `signal-analysis-rhythm`
  - onset, tempo, and beat tracking
- `signal-analysis-tonal`
  - chroma and key detection
- `signal-analysis-loudness`
  - LUFS, true peak (4x polyphase FIR), and LRA per BS.1770-4
- `signal-analysis-character`
  - spectral, temporal, and dynamics descriptor packs
- `signal-analysis-embed`
  - descriptor projection and tag matching

### Control plane

- `signal-runtime`
  - thin control library: lifecycle handshake, graph plan vocabulary, plugin
    discovery and sandbox lifecycle records, media decode/analysis pipeline,
    recording capture, and the supervisor/observation reports; control
    plane, not the audio callback (engine-block simulation, prework
    scheduler, and transport-session concurrency removed in g10.020)
- `signal-graph`
  - graph plan model (node specs, contracts, planning and contract
    summaries) for the control plane, never on the audio thread; the
    offline block-execution engine was removed in g10.020
- `signal-host-local`
  - Pulse-facing local host assembly (library crate; no binary)
- `signal-ipc`
  - shared-memory leases and the control/message model

### Plugin foundations

Real hosting for CLAP, VST3, AU, and LV2 through adapter crates,
`signal-plugin-sandbox`, and `signal-plugin-bridge`. Processing backends sit
behind the render-plane plugin handle (`RenderPluginProcessor`). Discovery
roots are explicit configuration defaulting empty.

- `signal-plugin`
  - format-neutral plugin types and host abstractions
- `signal-plugin-inventory`
  - shared plugin inventory domain for cross-product consumers
- `signal-plugin-clap`
  - CLAP discovery, lifecycle, `process()`, events, state, and GUI hosting
- `signal-plugin-vst3`
  - VST3 discovery, scan helper, lifecycle, `process()`, and GUI hosting
- `signal-plugin-au`
  - Audio Unit discovery (registry/plist), lifecycle, render pull, and GUI
    hosting
- `signal-plugin-lv2`
  - LV2 manifest scanning and lifecycle hosting
- `signal-plugin-sandbox`
  - out-of-process broker child over verified shm leases
- `signal-plugin-bridge`
  - host-side plugin processing backends: in-process and dedicated-sandbox
    tiers behind one placement-agnostic handle (shm round-trip with bounded
    wait and bypass-on-miss for full crash isolation)

## Current Audit Hotspots

- SharedSandbox tier (one broker, many plugins) landed in `g11.002`. Grouping
  key is `plugin:{plugin_type_id}`. Map:
  `docs/architecture/shared-sandbox-multiplexing.md`. DedicatedSandbox stays
  the default. Vendor/format grouping is out of v1.
- `signal-runtime`'s public surface is now pruned to its consumers
  (signal-host-local and pulse); anticipative rendering, when scheduled,
  re-derives against the render plane
  (`docs/architecture/prework-scheduler-design-note.md`)
- shared-memory broker permissions/cleanup hardening remains minimal

## Deferred Scope

- product-local UI shells, browser workflows, controller-page UX, and release
  packaging remain outside this inventory unless they are promoted into shared
  Signal-owned substrate

## Next Task

Keep inventory aligned with `docs/roadmaps/g11/README.md`. Execute
`docs/roadmaps/g11/batch-cards/005-g11-002-broker-multiplexing.md`.
