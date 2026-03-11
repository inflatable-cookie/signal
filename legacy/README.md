# Signal Legacy Surface

This folder holds the legacy C++ implementation surface for Signal.

It is retained as a reference and compatibility surface while the Rust
workspace becomes the primary active implementation.

## Current Layout

- `cpp/`
  - legacy C++ engine/runtime source tree
  - legacy tests
  - legacy CMake build surface
  - legacy build output now under `cpp/build/`

## Rule

- New shared DSP, graph, runtime, and trust-edge work should land in the Rust
  workspace under `crates/`.
- The C++ tree remains available for reference, validation, and staged
  migration work, but it is no longer the primary active repo surface.

## Next Task

Keep reducing the active root-facing importance of the legacy tree while
preserving enough buildability and traceability for reference-driven migration.
