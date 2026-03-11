# Roadmap g01.005: Core DSP Kernel and Control-Signal Baseline

Status: active
Owner: core-product
Created: 2026-03-10
Depends on: g01.004
Vision tags: RT, DSP, ENG
Target envelope: establish the first reusable realtime-safe DSP kernel layer so
graph, runtime, host, and plugin work can share stable low-level algorithms
instead of rebuilding utility code at every boundary.

## Problem

Signal now has a credible workspace layout, but the algorithmic foundation is
still too thin for serious engine work. Without a stronger `signal-dsp` and
`signal-primitives` baseline:

1. graph/runtime work will keep inventing one-off smoothing, buffer, and filter
   helpers locally,
2. plugin and hardware integration will arrive before the reusable kernel layer
   is stable,
3. migration from the legacy C++ engine will drift toward copy-shaped rewrites
   instead of a cleaner reusable DSP surface.

## Goals

- add stable control-rate primitives that can be reused across graph, runtime,
  plugin, and host code
- add a first concrete set of core DSP kernels for gain, filtering, delay, and
  level tracking
- define reset, bypass, denormal, and allocation rules for reusable kernels
- back the new kernels with deterministic numeric tests and reference fixtures

## Non-Goals

- building product-facing instruments or effects in this batch
- implementing heavy spectral or convolution processing in this batch
- binding these kernels to one specific host or plugin API before graph/runtime
  seams are ready

## Execution Plan

### 005.1 Control and primitive layer

- [x] audit `signal-primitives` and `signal-dsp` for missing low-level types
      needed by realtime kernels
- [x] add reusable smoothing and ramp primitives such as `SmoothedValue`,
      linear/exponential ramps, and sample-accurate step segments
- [x] add envelope/control helpers needed by parameter and transport-driven
      processing paths
- [x] define explicit reset/bypass contracts so stateful kernels can be reused
      safely by graph/runtime code

### 005.2 Core reusable kernels

- [x] add first reusable filter kernels with stable coefficient/configuration
      surfaces
- [x] add delay-line primitives with explicit capacity, tap, and feedback
      policies
- [x] add level/energy helpers such as peak, RMS, and envelope follower
      kernels for runtime diagnostics and metering reuse
- [x] add utility mix/sum helpers that avoid host-local buffer math drift

### 005.3 Numerical trust and migration references

- [x] add impulse, step, sine, and silence fixtures with tolerance-based tests
- [x] add regression coverage for denormal handling, reset behavior, and bypass
      continuity
- [x] identify which legacy C++ helpers remain reference-worthy and record the
      migration boundary in code comments or log evidence without copying the
      whole old utility surface
- [x] document the new kernel ownership rules in local package docs or milestone
      evidence so later engine work does not turn `signal-dsp` into a junk
      drawer

## Acceptance Signals

1. `signal-dsp` contains more than trivial demo kernels and is credible as the
   shared home for core reusable realtime-safe DSP utilities.
2. Graph/runtime work can depend on smoothing, filter, delay, and level kernels
   without open-coding them in higher layers.
3. The new algorithms are covered by deterministic numerical tests rather than
   only host-level smoke coverage.

## Risks and Mitigations

- Risk: the batch becomes an unbounded “implement all DSP” sweep.
- Mitigation: keep this milestone to foundational kernels only and defer
  specialized processors to later milestones when concrete runtime pressure
  appears.
- Risk: algorithm work gets copied mechanically from `legacy/cpp/`.
- Mitigation: treat the legacy tree as a behavioral reference only and prefer
  fresh Rust-native APIs with explicit realtime constraints.

## Evidence Requirements

- [ ] one log entry per meaningful DSP/kernel tranche under `docs/logs/YYYY-MM/`
- [ ] validation notes must include the algorithm-focused tests actually run
- [ ] any legacy-reference dependency called out explicitly in the closure log

## Next Task

Open `g01.006` once the core kernels are credible enough to route through a
real executable graph contract with deterministic routing, latency, and
parameter timing semantics.
