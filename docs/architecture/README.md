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
CLAP/VST3/AU/LV2 but in-process plugin hosting is future work. The stretch
headline is covered in [roadmaps](../roadmaps/README.md) — the transparent
renderer is frozen, and exact-ratio creative stretch (`Dream`, `Cyclic`) is
publicly admitted.

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
- `product-guardrails.md`
- `package-map.md`
- `dsp-analysis-feature-reference.md`
- `offline-time-stretch-synthesis.md`
- `offline-time-stretch-non-phase-vocoder-feasibility.md`
- `offline-creative-time-stretch-study.md`
- `offline-creative-continuous-direct-renewal-dream-brief.md`
- `offline-creative-cyclic-behavioral-synthesis.md`
- `offline-creative-centered-compressed-anchor-cyclic-brief.md`
- `offline-creative-event-ledger-audited-centered-compressed-anchor-cyclic-brief.md`
- `offline-creative-continuous-event-ledger-cyclic-brief.md`
- `offline-creative-fixed-ratio-public-surface.md`
- `offline-creative-audited-layered-cloud-brief.md`
- `offline-creative-layered-cloud-brief.md`
- `offline-creative-linked-stn-noise-morph-brief.md`
- `offline-creative-verified-source-relative-renewal-spectral-brief.md`
  - frozen `SupportAuditedListeningLedSourceRelativeRenewalSpectral` authority
- `offline-creative-renewal-spectral-brief.md`
- `offline-creative-continuous-excitation-complex-relation-brief.md`
- `offline-creative-continuous-excitation-spectral-brief.md`
- `offline-creative-diffuse-spectral-brief.md`
- `offline-time-stretch-successor-brief.md`
- `graph-runtime-feature-reference.md`
- related contracts under `docs/contracts/`

## Next Task

Keep the `g10.030` transparent successor program closed. The exact-ratio
Dream wrapper is admitted and `g10.031` is complete. `g10.032` privately
admits the accepted event-ledger Cyclic renderer and publicly admits its
fixed-ratio extension; Batch 32.29 closes that lane. `g10.033` admits one
continuous `4x..16x` Dream owner and its direct public surface without
routing. Batch 33.6 publishes the exact executable matrix and closes the lane.
`g10.034` Batch 34.2 freezes one complete `2N..=8N` Cyclic candidate with
exact anchor parity, interior acoustic evidence, and cleanup. Execute Batch
34.3 in one disposable worktree; keep public widening closed.
