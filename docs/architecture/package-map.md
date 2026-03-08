# Package Map

Status: active
Owner: core-product
Updated: 2026-03-08
Vision refs: `docs/vision/001-signal-vision.md`
Architecture refs: `docs/architecture/system-architecture.md`

## Purpose

Freeze the first naming proposal for the extracted Signal workspace so research,
architecture, and implementation can converge on stable package and host names.
The Rust workspace now lives under `signal/crates/`, so package names remain
stable while their on-disk layout is grouped under one explicit workspace root.

The main naming rule is:

- use `signal-<layer>` or `signal-<layer>-<domain>`
- prefer broad, reusable domain names over Finch-oriented feature names
- avoid vague buckets such as `signal-core` unless the content is truly
  irreducibly generic

## Naming Principles

1. `signal-primitives` is better than `signal-core`
   - `core` becomes a junk drawer
   - `primitives` makes the boundary explicit
2. `signal-analysis-rhythm` is better than `signal-beat`
   - the domain is larger than beat positions
   - onset, tempo, groove, meter, and confidence belong together
3. `signal-dsp-spectral` is better than `signal-spectral`
   - spectral work is a DSP layer, not a product-facing domain by itself
4. keep host-edge crates visibly separate from reusable DSP crates
   - plugin and hardware crates are integration boundaries, not algorithm homes

## Recommended Workspace Surface

### 1. Foundation

- `signal-primitives`
  - sample/frame/time/channel types
  - buffer primitives
  - realtime-safe utility types
- `signal-params`
  - parameter descriptors
  - smoothing/event primitives
  - modulation-facing parameter utilities
- `signal-midi`
  - MIDI event/model primitives
  - message normalization and routing helpers
- `signal-io`
  - audio decode/encode and probe surfaces
  - file/container helpers
  - shared offline asset loading

### 2. DSP

- `signal-dsp`
  - generic DSP kernels
  - filters, envelopes, dynamics helpers, metering primitives
- `signal-dsp-spectral`
  - FFT/STFT windows
  - spectral transforms and low-level spectral features
- `signal-dsp-resample`
  - sample-rate conversion
  - rate/timebase helpers

### 3. Analysis

- `signal-analysis`
  - shared analysis result types
  - confidence model
  - streaming/offline analysis traits
- `signal-analysis-rhythm`
  - onset detection
  - tempo estimation
  - beat tracking
  - meter/groove follow-ons
- `signal-analysis-tonal`
  - chroma extraction glue
  - key detection
  - tuning estimation
  - future chord/harmonic follow-ons
- `signal-analysis-loudness`
  - LUFS
  - true peak
  - loudness-range and related dynamics measurements
- `signal-analysis-embed`
  - embedding inference
  - future classifier support

### 4. Execution

- `signal-graph`
  - graph model and execution semantics
  - routing, latency, tail, scheduling interfaces
- `signal-runtime`
  - embeddable engine/runtime orchestration
  - transport-facing runtime state
  - runtime-owned block sequencing and continuity tracking across lease rollover
  - runtime-owned supervision, watchdog escalation, and readiness degradation
  - shared supervisor report types, including runtime-owned timeline and automation snapshots
  - diagnostics and readiness surfaces
- `signal-supervisor-tools`
  - live runtime supervisor and soak-reporting CLI
  - real host scenario inspection outside the host `main` binaries
  - versioned text and machine-readable export for soak and restart scenarios
  - contract-bound export envelope for automation and external tooling
- `signal-ipc`
  - runtime control protocol
  - message/event model shared across hosts and consumers

### 5. Trust-Edge Integration

- `signal-plugin`
  - plugin-host abstractions
  - common instance/state/parameter surfaces
- `signal-plugin-clap`
  - CLAP adapter
- `signal-plugin-vst3`
  - VST3 adapter
- `signal-plugin-au`
  - AU adapter
- `signal-hardware`
  - common audio/MIDI device abstractions
  - device model and diagnostics contracts
- `signal-hardware-coreaudio`
  - macOS backend
- `signal-hardware-wasapi`
  - Windows shared/exclusive backend

Linux backends can be added later only when real implementation pressure
appears:

- `signal-hardware-alsa`
- `signal-hardware-jack`

### 6. Host Assemblies

- `signal-host-local`
  - local desktop runtime host
- `signal-host-server`
  - headless/remote runtime host
- `signal-plugin-sandbox`
  - out-of-process plugin container

## What I Would Not Freeze As Long-Term Names

- `signal-core`
  - too vague; likely to become a dumping ground
- `signal-beat`
  - too narrow and too Finch-shaped
- `signal-tonal`
  - workable, but weaker than the consistent `signal-analysis-*` family
- `signal-loudness`
  - same issue; better grouped under analysis
- `signal-spectral`
  - spectral transforms belong in the DSP layer

## First Concrete Freeze

If we want the smallest useful first batch, I would freeze these names first:

- `signal-primitives`
- `signal-io`
- `signal-dsp`
- `signal-dsp-spectral`
- `signal-analysis`
- `signal-analysis-rhythm`
- `signal-analysis-tonal`
- `signal-analysis-loudness`
- `signal-graph`
- `signal-runtime`
- `signal-plugin`
- `signal-plugin-clap`
- `signal-hardware`
- `signal-host-local`
- `signal-host-server`
- `signal-plugin-sandbox`

Then add these only when implementation pressure justifies them:

- `signal-params`
- `signal-midi`
- `signal-dsp-resample`
- `signal-analysis-embed`
- `signal-plugin-vst3`
- `signal-plugin-au`
- `signal-hardware-coreaudio`
- `signal-hardware-wasapi`

## Current Workspace State

These packages now exist as real workspace members under `crates/`:

- `signal-primitives`
- `signal-dsp`
- `signal-dsp-spectral`
- `signal-analysis`
- `signal-analysis-rhythm`
- `signal-analysis-tonal`
- `signal-analysis-loudness`
- `signal-graph`
- `signal-runtime`
- `signal-ipc`
- `signal-plugin`
- `signal-plugin-clap`
- `signal-plugin-sandbox`
- `signal-hardware`
- `signal-hardware-coreaudio`
- `signal-host-local`
- `signal-host-server`

These names should be treated as frozen implementation targets unless a later
architecture batch explicitly changes them.

## Layout Note

The Rust workspace packages now live under:

```text
signal/
  crates/
    signal-primitives/
    signal-dsp/
    signal-dsp-spectral/
    signal-analysis/
    signal-analysis-rhythm/
    signal-analysis-tonal/
    signal-analysis-loudness/
    signal-graph/
    signal-runtime/
    signal-ipc/
    signal-plugin/
    signal-plugin-clap/
    signal-plugin-sandbox/
    signal-hardware/
    signal-hardware-coreaudio/
    signal-host-local/
    signal-host-server/
    signal-supervisor-tools/
```

This keeps the repository root reserved for repo-level concerns such as the
legacy C++ implementation, docs, top-level build surfaces, and workspace
manifests.

## Next Task

Decide whether the payload-only debug policy is now sufficiently frozen to
leave this export boundary alone for a while, or whether there is a concrete
inspection need strong enough to justify a second explicit debug section.
