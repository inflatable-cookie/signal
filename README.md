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

# Shared audio-systems runtime and DSP workspace

Signal is a standalone, general-purpose realtime audio library. The Loophole
DAW is its primary consumer, but Signal is *for* Loophole, not *owned* by it:
it provides mechanism (typed channel formats, graph execution, honest
realtime boundaries) and leaves mixing/layout policy to consuming
applications.


Signal is the shared audio-systems repo for Loophole, Finch, and future apps.
Its active surface is the Rust library/runtime workspace under `crates/`.

Signal may run out-of-process where isolation is the right trade, but the repo
itself is no longer defined by one mandatory standalone process topology.

## Responsibilities

Signal is responsible for:

- Real-time audio output: alloc-free render execution on negotiated device
  streams (the production path today)
- DSP kernels and offline audio analysis (rhythm, tonal, loudness, character)
- Plugin discovery and cataloguing (CLAP/VST3/AU/LV2); in-process hosting is
  a future rebuild program, not a current surface
- Offline render orchestration and runtime diagnostics (control plane)

Signal is not responsible for project editing/state ownership (Pulse) or UI
behavior (Aura/Spark/Finch UI).

## Current Repository Layout

Production audio path first, then analysis, control plane, and plugin
foundations. Every workspace crate is listed.

```
crates/
  # Production audio path
  signal-render-plane/         # Alloc-free realtime executor: compiled plans, declick
  #                              envelopes, polyphase clip resampling on the audio thread
  signal-hardware/             # Output stream contract: specs, negotiation types, device model
  signal-hardware-cpal/        # cpal-backed negotiated input/output streams + real device enumeration

  # DSP substrate
  signal-primitives/           # Shared sample/frame/buffer/time primitives
  signal-dsp/                  # DSP kernels: ramps, smoothing, delay, polyphase
  #                              interpolation table (the RT-path resampler)
  signal-dsp-spectral/         # FFT/STFT windows and spectral transforms
  signal-dsp-resample/         # Offline/streaming mono resampler for analysis input prep

  # Analysis
  signal-analysis/             # Shared analysis traits, result types, input prep
  #                              (corpus/acceptance harness behind `test-support` feature)
  signal-analysis-rhythm/      # Onset, tempo, beat tracking
  signal-analysis-tonal/       # Chroma and key detection
  signal-analysis-loudness/    # LUFS, true peak (4x polyphase), LRA per BS.1770
  signal-analysis-character/   # Spectral/temporal/dynamics descriptor packs
  signal-analysis-embed/       # Descriptor projection and tag matching

  # Control plane
  signal-runtime/              # Thin control library: lifecycle, graph plan vocabulary,
  #                              plugin discovery/sandbox records, media pipeline,
  #                              observation reports (not the audio callback)
  signal-graph/                # Graph plan model (specs, contracts, planning summaries)
  #                              for the control plane, never on the audio thread
  signal-host-local/           # Pulse-facing local host assembly (library; no binary)
  signal-ipc/                  # Shared-memory leases and control/message model

  # Plugin foundations (discovery/catalog, no in-process hosting)
  signal-plugin/               # Format-neutral plugin types and host abstractions
  signal-plugin-inventory/     # Shared plugin inventory domain for consumers
  signal-plugin-clap/          # CLAP discovery via real clap-sys/libloading FFI
  signal-plugin-vst3/          # VST3 discovery and COM introspection
  signal-plugin-au/            # Audio Unit discovery with plist pre-filter
  signal-plugin-lv2/           # LV2 manifest scanning
  signal-plugin-sandbox/       # Out-of-process plugin container shell (shm broker)
docs/                          # Vision, architecture, contracts, roadmaps, logs
```

Discovery roots are explicit configuration and default empty; no adapter
scans system plugin directories unless told to.

## Development

Use Effigy as the default command surface inside `signal/`:

```bash
effigy tasks
effigy doctor
effigy health
effigy validate
effigy qa:docs
```

Rust workspace directly:

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all --check
cargo run -p signal-hardware-cpal --example tone
cargo run -p signal-render-plane --example render_soak
```

CI (`.github/workflows/ci.yml`) enforces build, test, fmt, and clippy on
every push and pull request. The hardware smoke tests self-skip when no
output device is present.

## Documentation

Use the local docs bundle for architecture, research, roadmaps, and logs:

```bash
open docs/README.md
```

Key entry points:

- `docs/vision/001-signal-vision.md`
- `docs/architecture/system-architecture.md`
- `docs/contracts/001-shared-dsp-and-host-boundary.md`
- `docs/research/master-index.md`

## Real-Time Safety

Real-time code paths must avoid allocation, blocking calls, lock contention, and unbounded work. Treat plugin code as untrusted and keep API boundaries defensive.

## Licence

Signal is provided under the MIT Licence with the following additional clause:

**The Loophole name (including its components: Signal, Pulse, Aura and Chorus)
may not be used to promote or endorse any derived product without prior written
permission from the copyright holder.**

This clause applies to all repositories within the Loophole ecosystem.
