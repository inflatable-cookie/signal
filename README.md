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

Signal is the shared audio-systems repo for Loophole, Finch, and future apps.
It currently contains the legacy C++ engine/runtime implementation, and it will
also become the home of the shared Rust DSP, analysis, graph, and runtime
components that those products reuse.

Signal may run out-of-process where isolation is the right trade, but the repo
itself is no longer defined by one mandatory standalone process topology.

## Responsibilities

Signal is responsible for:

- Real-time audio processing
- MIDI input/output handling
- Runtime plugin backend integration (VST3/CLAP backends in-tree)
- Processing graph execution
- Sample-accurate timing-sensitive engine behavior
- Engine telemetry and diagnostics emission

Signal is not responsible for project editing/state ownership (Pulse) or UI
behavior (Aura/Spark/Finch UI).

## Current Repository Layout

```
crates/
  signal-primitives/        # Shared sample/frame/buffer/time primitives
  signal-dsp/               # General reusable DSP kernels
  signal-dsp-spectral/      # FFT/STFT and spectral transforms
  signal-analysis/          # Shared analysis traits and result types
  signal-analysis-rhythm/   # Onset, tempo, beat, meter
  signal-analysis-tonal/    # Chroma, tuning, key, harmonic follow-ons
  signal-analysis-loudness/ # LUFS, true peak, LRA
  signal-graph/             # Graph model and execution semantics
  signal-runtime/           # Embeddable runtime orchestration
  signal-ipc/               # Shared runtime control/message seam
  signal-plugin/            # Format-neutral plugin abstractions
  signal-plugin-clap/       # CLAP adapter shell
  signal-plugin-sandbox/    # Out-of-process plugin container shell
  signal-hardware/          # Common device abstractions
  signal-hardware-coreaudio/# CoreAudio backend shell
  signal-host-local/        # Local desktop runtime host shell
  signal-host-server/       # Headless runtime host shell
  signal-supervisor-tools/  # Live supervisor and soak-reporting CLI
src/
  backend/          # Engine runtime/backend glue
  clap/             # CLAP backend integration
  core/             # Core engine primitives
  domains/          # Domain-level command handling
  ipc/              # Envelope and domain codecs
  logging/          # Logging and diagnostics plumbing
  vst3/             # VST3 backend integration
tests/              # Engine tests
docs/               # Local docs/spec notes
CMakeLists.txt
Cargo.toml
```

Northstar-aligned planning and research docs now live under `docs/`.

## Development

Use Effigy as the default command surface inside `signal/`:

```bash
effigy tasks --repo .
effigy health --repo .
effigy dev --repo .
effigy validate --repo .
```

Equivalent raw CMake flow:

```bash
cmake -S . -B build
cmake --build build --config Debug
ctest --test-dir build --output-on-failure
```

Rust workspace bootstrap:

```bash
cargo check --workspace
cargo run -p signal-host-local
cargo run -p signal-supervisor-tools -- --describe-export --format=json
cargo run -p signal-supervisor-tools -- --format=json local soak
cargo run -p signal-supervisor-tools -- --format=json --include-payload local soak
```

The supervisor tool now emits a versioned JSON export schema with both
host-derived execution/transport/fault summaries and the shared runtime
supervisor report. Payload detail is excluded by default and can be added
explicitly with `--include-payload` for debugging and soak inspection. The
`host_summary` export also declares its included section list so automation can
distinguish default exports from payload-augmented debug runs without relying
on implicit shape assumptions, and it now also declares both supported and
enabled debug sections so the current payload-only debug policy is explicit in
the export itself. Use `--describe-export` when tooling needs that frozen
export policy without booting a host scenario.

All Rust workspace packages now live under `crates/`. Keep new Rust packages
under that directory rather than adding more top-level package folders.

Current trust-edge workspace shells:

- `signal-ipc`
- `signal-plugin`
- `signal-plugin-clap`
- `signal-plugin-sandbox`
- `signal-hardware`
- `signal-hardware-coreaudio`
- `signal-host-local`
- `signal-host-server`
- `signal-supervisor-tools`

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
