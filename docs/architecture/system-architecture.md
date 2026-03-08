# System Architecture

Status: active
Owner: core-product
Updated: 2026-03-08
Vision refs: `docs/vision/001-signal-vision.md`

## Top-Level Stack

Signal owns the shared audio-systems stack used by Loophole, Finch, and future
apps.

The intended top-level layers are:

1. `signal-primitives`
   - audio/sample/time/channel primitives
   - realtime-safe math and utility types
2. `signal-dsp`
   - reusable DSP kernels and transforms
   - smoothing, metering, resampling, filters
3. `signal-analysis`
   - onset, beat, tempo, tonal, loudness, and future embedding-related analysis
   - reusable offline and streaming analysis logic
4. `signal-graph`
   - graph execution semantics
   - routing, latency/tail accounting, parameter-event application
5. `signal-runtime`
   - embeddable runtime orchestration
   - diagnostics, scheduling, lifecycle, and host-facing runtime state
6. host-edge adapters
   - plugin-format adapters
   - hardware/device adapters
   - narrow FFI or IPC boundaries only where platform reality forces them

The current in-repo C++ engine remains a temporary compatibility island while
the Rust-owned shared stack is built out.

The current package-level naming proposal is recorded in
`docs/architecture/package-map.md`.

## Data and Authority Flow

- Signal owns audio execution, DSP, analysis, graph/runtime semantics, and
  runtime diagnostics.
- Pulse remains the authority for Loophole project/session state and editing.
- Finch remains the authority for app workflow, review UX, sidecar handling, and
  library-specific behavior.
- Finch and Loophole both consume Signal-owned crates or runtime surfaces rather
  than reimplementing core analysis logic locally.

## Invariants

- Reusable DSP and analysis do not live in Finch-local or Loophole-local wrapper
  code.
- Plugin and hardware integrations must not become the home of core DSP logic.
- Process boundaries follow trust and stability needs, not historical repo
  layout.
- Supervisor export contracts prefer shared runtime report surfaces over
  host-specific summary duplication.
- Real-time paths avoid blocking, allocation churn, and unbounded work.
- Research authority for DSP and analysis topics lives in `docs/research/`.

## Supervisor Export Boundary

- `signal-runtime` owns the shared supervisor report types and any continuity
  state that has been promoted into runtime-owned surfaces.
- Host assemblies may expose convenience summaries, but runtime-owned continuity
  state should be sourced from `signal-runtime` rather than recomputed locally.
- `signal-supervisor-tools` is the versioned export boundary for machine-readable
  soak and restart reporting.
- The explicit export and report rules live in
  `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`.

## Performance and Reliability Constraints

- Realtime-safe code paths must be deterministic and allocation-aware.
- Shared crates must remain usable in both runtime and offline-analysis
  contexts.
- Plugin hosting is treated as untrusted by default; sandboxing remains the
  preferred containment layer.
- Native shims are acceptable where ABI or platform constraints make them the
  lower-risk integration choice.

## Interfaces With Roadmaps

- `g01.001` establishes the Signal docs authority and migrates DSP research into
  this repo.
- Follow-on milestones should freeze crate names, host entrypoints, and the
  first reusable Rust implementation slices.

## Next Task

Use the package map to lock the first Signal-owned package names, then define
the migration boundary between the current C++ runtime and the new shared Rust
crates.
