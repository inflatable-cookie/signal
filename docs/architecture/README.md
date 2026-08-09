# Architecture

Status: active
Updated: 2026-07-24

## In plain words

Signal is a reusable audio library, not an app. This section explains how the
pieces fit together: the crate boundaries, the runtime shape, and the
invariants every seam has to respect. If you want to know *what exists today*,
read the [system architecture](./system-architecture.md) and the
[package map](./package-map.md). If a term is unfamiliar, see the
[glossary](../reference/glossary.md).

Current headline state: the realtime render path, DSP kernels, analysis, graph,
and runtime crates are live under `crates/`; plugin discovery exists for
CLAP/VST3/AU/LV2, with processing backends behind the render-plane plugin
handle not yet wired into a host assembly. The stretch headline is covered in
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
- stretch decision and brief files: `offline-time-stretch-non-phase-vocoder-feasibility.md`,
  `offline-creative-time-stretch-study.md`, and the `offline-creative-*`
  briefs under this directory (admitted and rejected candidates alike —
  rejected briefs are retained as evidence)
- related contracts under `docs/contracts/`

## Next Task

Keep the `g10` stretch state closed: Transparent, Dream, and Cyclic are
explicit, admitted characters; Automatic is closed; `RealtimePreview` is
proven and unadopted. No batch is ready — return to
`docs/roadmaps/g10/README.md` for an operator-selected Signal-only target.
