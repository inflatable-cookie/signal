# Architecture

Status: active
Updated: 2026-08-17

## In plain words

Signal is a reusable audio library, not an app. This section explains how the
pieces fit together: the crate boundaries, the runtime shape, and the
invariants every seam has to respect. If you want to know *what exists today*,
read the [system architecture](./system-architecture.md) and the
[package map](./package-map.md). If a term is unfamiliar, see the
[glossary](../reference/glossary.md).

Current headline state: the realtime render path, DSP kernels, analysis, graph,
and runtime crates are live under `crates/`; CLAP, VST3, AU, and LV2 hosting
is implemented through `signal-plugin-bridge` with production host-assembly
wiring in `signal-host-local` (`g11.001`). SharedSandbox multiplexing landed
in `g11.002`. The stretch headline is covered in
[roadmaps](../roadmaps/README.md) — the transparent renderer is frozen and
corrected through `g10.042`, and exact-ratio creative stretch (`Dream`,
`Cyclic`) is publicly admitted.

## Why this section matters now

Architecture defines Signal as a reusable library system rather than a
Loophole-specific engine app.

## Scope

Use this section for:

- crate and package boundaries
- embeddable runtime shape
- trust-edge adapter boundaries
- generic library invariants

Keep milestone sequencing in `roadmaps/`.

## Active Entry Points

- `system-architecture.md`
- `system-inventory.md` — the authoritative crate list
- `product-guardrails.md`
- `dsp-analysis-feature-reference.md`
- `graph-runtime-feature-reference.md`
- `offline-time-stretch-synthesis.md`
- `offline-creative-fixed-ratio-public-surface.md`
- [Production Host-Assembly Integration](./production-host-assembly-integration.md)
- [SharedSandbox Multiplexing](./shared-sandbox-multiplexing.md)
- stretch decision and brief files: `offline-time-stretch-non-phase-vocoder-feasibility.md`,
  `offline-creative-time-stretch-study.md`, and the `offline-creative-*`
  briefs under this directory (admitted and rejected candidates alike —
  rejected briefs are retained as evidence)
- related contracts under `docs/contracts/`

## Next Task

Stop for operator selection of the next Signal-only backlog pull. Do not start
a follow-on generation. `g11.001` and `g11.002` are complete. Keep the `g10`
stretch state closed. Linux CLAP filesystem discovery (`086`) shipped
2026-08-21. Do not open `g12`.
