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

# Real-time audio engine server

Signal is the dedicated audio-engine process for the Loophole DAW. It is implemented in C++ (CMake, C++20) and is responsible for real-time audio/MIDI processing, plugin runtime integration, and engine telemetry.

Signal runs out-of-process for isolation from UI and project-model concerns.

## Responsibilities

Signal is responsible for:

- Real-time audio processing
- MIDI input/output handling
- Runtime plugin backend integration (VST3/CLAP backends in-tree)
- Processing graph execution
- Sample-accurate timing-sensitive engine behavior
- Engine telemetry and diagnostics emission

Signal is not responsible for project editing/state ownership (Pulse) or UI behavior (Aura/Spark).

## Current Repository Layout

```
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
```

## Development

Signal uses CMake and exposes convenience scripts through `signal/package.json`:

```bash
# Build debug artifacts
bun run build

# Build + run signal binary
bun run dev

# Build + run CTest suite
bun run test
```

Equivalent raw CMake flow:

```bash
cmake -S . -B build
cmake --build build --config Debug
ctest --test-dir build --output-on-failure
```

## Real-Time Safety

Real-time code paths must avoid allocation, blocking calls, lock contention, and unbounded work. Treat plugin code as untrusted and keep API boundaries defensive.

## Licence

Signal is provided under the MIT Licence with the following additional clause:

**The Loophole name (including its components: Signal, Pulse, Aura and Chorus)
may not be used to promote or endorse any derived product without prior written
permission from the copyright holder.**

This clause applies to all repositories within the Loophole ecosystem.
