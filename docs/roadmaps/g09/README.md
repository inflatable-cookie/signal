# g09 Milestones

Status: active
Updated: 2026-04-09

## Why this generation matters now

`g08` closed the feature-expansion and live-ownership runway, but the audit
that followed exposed a different kind of bottleneck: too much of Signal's
claimed breadth is still contract-deep but implementation-thin, while several
core runtime and analysis surfaces remain overly coupled, panic-prone, or too
implicit in their failure behavior.

`g09` therefore shifts from feature breadth to audit-driven realization and
proof:

- replace scaffolded plugin and native-backend implementations with real
  discovery, execution, and device truth
- remove duplicated host recovery and execution logic
- decompose the remaining oversized runtime public surfaces
- harden low-level correctness, protocol safety, and failure explicitness
- modernize fidelity-sensitive DSP and rhythm substrate
- add interactive demos as repo-owned capability proof for the active crate set

## Dependency order

1. freeze the audit-remediation planning and contract baseline first
2. realize shared plugin-hosting substrate before format-specific depth
3. unify host/runtime behavior before widening more host-facing features
4. decompose runtime public surfaces before adding more downstream consumers
5. harden low-level and fidelity debt before demo proof tries to showcase it
6. build the shared demo substrate before domain demos
7. close with demo and remediation proof rather than prose-only completion

## Milestone map

- `g09.001` `complete`
  - audit planning surfaces and remediation contract freeze
- `g09.002` `complete`
  - shared plugin-hosting substrate and hardened sandbox execution
- `g09.003` `complete`
  - real VST3 discovery, instantiation, and lifecycle proof
- `g09.004` `complete`
  - real AU discovery plus CoreAudio-backed macOS device truth
- `g09.005` `complete`
  - real LV2 discovery, extension negotiation, and Linux proof
- `g09.006` `complete`
  - shared host/runtime execution and recovery unification
- `g09.007` `complete`
  - runtime interface decomposition and test-surface normalization
- `g09.008` `active`
- `g09.008` `complete`
  - low-level correctness, safety, and protocol hardening
- `g09.009` `active`
  - DSP fidelity and semantic-analysis realism uplift
- `g09.010` `draft`
  - rhythm-engine resilience and policy normalization
- `g09.011` `draft`
  - interactive demo substrate, manifest, and operator conventions
- `g09.012` `draft`
  - host/runtime/plugin/hardware interactive demo suite
- `g09.013` `draft`
  - DSP/graph/analysis interactive demo suite and audit closeout proof

## Lane structure

### Lane A - Real Plugin And Device Ownership

`001 -> 002 -> 003 -> 004 -> 005`

Replace scaffolding with real adapter, sandbox, and native-backend
implementations.

### Lane B - Runtime And Host Structural Repair

`001 -> 006 -> 007 -> 008`

Remove duplicated host behavior, shrink the runtime public wall, and harden the
low-level substrate.

### Lane C - Fidelity And Analysis Repair

`001 -> 009 -> 010`

Raise quality and resilience in the DSP, semantic, and rhythm hot paths.

### Lane D - Executable Proof

`001 -> 011 -> 012 -> 013`

Turn crate claims into repo-owned interactive demos and close the generation
with proof instead of prose.

## Strict lane attachment

`g09` is now carrying a lane-first strict Northstar surface.

- strict-lane spec:
  `docs/specs/001-g09-lane-first-strict-adoption.md`
- current ready card:
  `docs/specs/batch-cards/010-g09-009-resampler-proof-and-benchmark-surface.md`

## Working rules for this thread

- keep one active queue under `g09`
- treat legacy C++ as reference only unless explicitly reactivated
- prefer contract-backed breaking changes over compatibility shims
- keep real-time paths allocation-safe and explicit about degraded behavior
- do not claim a crate capability through demo or roadmap prose unless one of:
  - production code implements it, or
  - the roadmap explicitly records it as deferred

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/010-g09-009-resampler-proof-and-benchmark-surface.md`.
