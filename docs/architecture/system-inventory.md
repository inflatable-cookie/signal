# System Inventory

Status: active
Owner: core-product
Updated: 2026-04-08
Vision refs: `docs/vision/001-signal-vision.md`

## Purpose

Record the execution-relevant Signal surface so roadmap and contract work can
sequence against explicit crate, boundary, and proof ownership instead of
implicit repo context.

## Workspace Scope

Signal's active implementation surface is the Rust workspace under `crates/`.
The legacy C++ tree under `legacy/cpp/` remains reference material only unless
explicitly promoted back into active planning.

## Layer Inventory

### Foundation and DSP substrate

- `signal-primitives`
  - core sample, frame, transport, and channel-layout types
  - weak invariant risk remains around zero-count layouts and lossy
    interleaved-buffer construction
- `signal-dsp`
  - reusable control, automation, and DSP kernels
- `signal-dsp-resample`
  - deterministic nearest and linear resampling substrate
  - currently lacks higher-fidelity anti-aliasing and quality-mode selection

### Analysis substrate

- `signal-analysis-rhythm`
  - onset, beat, meter, and tempo continuity substrate
  - currently carries duplicated state-policy arms and panic-on-worker-failure
    risk
- `signal-analysis-tonal`
  - tonal profile, key, ambiguity, and tuning analysis
- `signal-analysis-loudness`
  - loudness, gating, weighting, range, and trace support
- `signal-analysis-character`
  - temporal, transient, dynamics, and descriptor-oriented character analysis
- `signal-analysis-embed`
  - descriptor embedding and semantic-tag scoring
  - currently heuristic-weight driven rather than calibrated-model backed

### Graph and runtime substrate

- `signal-graph`
  - executable graph topology, routing, buses, execution planning, and stage
    processing
  - currently allows silent zeroed-buffer adaptation for unsupported layout
    mismatches
- `signal-runtime`
  - embeddable runtime shell, interfaces, lifecycle, scheduling, diagnostics,
    receipts, and recovery policy
  - still holds the largest remaining coupled public contract surface

### Plugin, host, and hardware edge

- `signal-plugin`
  - format-neutral plugin contracts, model, sandbox protocol, and lifecycle
    vocabulary
- `signal-plugin-inventory`
  - shared plugin inventory domain for cross-product consumers
  - currently bootstrap-level and not yet wired into runtime-owned discovery
- `signal-plugin-library`
  - canonical plugin library domain for cross-product organization semantics
  - currently bootstrap-level and not yet adopted by downstream consumers
- `signal-plugin-library-store`
  - storage traits and mutation batch seam for shared plugin inventory/library
    consumers
  - currently defines trait boundaries only, not a shared concrete adapter
- `signal-plugin-clap`
  - CLAP-specific adapter and sandbox harness realization
  - currently still contains panic-oriented `expect(...)` paths inside request
    handling
- `signal-plugin-vst3`
  - VST3 adapter crate
  - current discovery path is scaffolded and not yet real module/class
    introspection
- `signal-plugin-au`
  - AU adapter crate
  - current discovery and lifecycle path is scaffolded and not yet real
    AudioComponent realization
- `signal-plugin-lv2`
  - LV2 adapter crate
  - current discovery path is scaffolded and not yet real manifest/bundle
    traversal
- `signal-plugin-sandbox`
  - sandbox process crate
  - currently demo-harness oriented rather than a hardened long-lived
    production broker
- `signal-hardware`
  - backend-neutral hardware model and simulation surface
- `signal-hardware-coreaudio`
  - macOS backend crate
  - currently simulated/default-device oriented rather than real CoreAudio
    ownership
- `signal-host-local`
  - in-process host assembly over runtime
- `signal-host-server`
  - server-style host assembly over runtime
  - local and server hosts still duplicate substantial execution and recovery
    policy

### IPC, tooling, and proof surfaces

- `signal-ipc`
  - runtime/plugin transport protocol and shared-memory broker
  - current shared-memory lifecycle and permission hardening remain minimal
- `signal-supervisor-tools`
  - machine-readable acceptance, export, and grouped proof descriptors
- `effigy`
  - repo-owned validation, task, doctor, and acceptance front door

## Current Audit Hotspots

### Product-facing realization gaps

- plugin hosting is contract-rich but implementation-thin across VST3, AU, LV2,
  sandbox bring-up, and CoreAudio device ownership
- host-local and host-server still replicate too much runtime-block and recovery
  behavior
- interactive demo and proof binaries are not yet a first-class repo-owned
  surface for crate claims

### Structural design debt

- `signal-runtime` still exposes oversized, multi-family public contract roots
- `signal-runtime` test trees still depend on large shared fixture/import walls
- `signal-analysis-rhythm` tempo and meter state policy remains branch-heavy and
  difficult to evolve safely

### Low-level correctness and safety debt

- graph bus mismatch handling silently zeroes buffers instead of surfacing
  explicit contract failure
- CLAP harness request handling can still panic on internal-state mismatch
- shared-memory broker ownership, cleanup, and permissions are under-specified
- primitive buffer/layout constructors permit weak or lossy state

### Fidelity and realism debt

- resampling quality is intentionally minimal
- semantic embedding remains heuristic and hand-tuned
- rhythm worker failures still crash the path instead of degrading cleanly

## Planning Implications

- Roadmap work after `g08` should no longer optimize for breadth-first feature
  expansion.
- The next generation should concentrate on:
  - replacing scaffolded adapter and backend implementations with real runtime
    ownership
  - decomposing oversized runtime and test surfaces
  - hardening low-level correctness and protocol safety
  - modernizing the rhythm and analysis fidelity hot paths
  - adding interactive demos as repo-owned capability proof, not optional
    polish

## Deferred Scope

- legacy C++ modernization stays outside active planning unless explicitly
  reactivated
- product-local UI shells, browser workflows, controller-page UX, and release
  packaging remain outside this inventory unless they are promoted into shared
  Signal-owned substrate

## Next Task

Use this inventory as the execution-relevant front door for the new post-`g08`
audit-remediation generation, then keep it aligned with any new contracts and
roadmaps opened under `docs/roadmaps/g09/`.
