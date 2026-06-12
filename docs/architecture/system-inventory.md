# System Inventory

Status: active
Owner: core-product
Updated: 2026-06-11
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
(24 crates).

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

Discovery and cataloguing only; in-process hosting is a future rebuild
program. Discovery roots are explicit configuration defaulting empty.

- `signal-plugin`
  - format-neutral plugin types and host abstractions
- `signal-plugin-inventory`
  - shared plugin inventory domain for cross-product consumers
- `signal-plugin-clap`
  - CLAP discovery via real `clap-sys`/`libloading` FFI;
    factory-descriptor-only by default, instance probing opt-in
- `signal-plugin-vst3`
  - VST3 discovery and COM introspection
- `signal-plugin-au`
  - Audio Unit discovery with plist pre-filter
- `signal-plugin-lv2`
  - LV2 manifest scanning
- `signal-plugin-sandbox`
  - out-of-process plugin container shell over verified shm leases

## Current Audit Hotspots

- plugin hosting (instantiate/process) does not exist yet; rebuild-on-demand
  items live in `docs/roadmaps/backlog/post-g10-rebuild-on-demand.md`
- `signal-runtime`'s public surface is now pruned to its consumers
  (signal-host-local and pulse); anticipative rendering, when scheduled,
  re-derives against the render plane
  (`docs/architecture/prework-scheduler-design-note.md`)
- shared-memory broker permissions/cleanup hardening remains minimal
- `signal-plugin-clap` discovery FFI defers per-operation unsafe blocks to
  the CLAP hosting rebuild (`unsafe_op_in_unsafe_fn` allowed crate-wide
  with reason)

## Deferred Scope

- product-local UI shells, browser workflows, controller-page UX, and release
  packaging remain outside this inventory unless they are promoted into shared
  Signal-owned substrate

## Next Task

Keep this inventory aligned with the g10 continuation packets
(`docs/roadmaps/g10/`) and any rebuild-on-demand work pulled from the
post-g10 backlog.
