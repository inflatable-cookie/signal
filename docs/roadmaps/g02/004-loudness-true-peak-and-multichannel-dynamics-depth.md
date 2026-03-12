# Roadmap g02.004: Loudness, True-Peak, and Multichannel Dynamics Depth

Status: complete
Owner: core-product
Created: 2026-03-11
Depends on: g02.001
Vision tags: RES, DSP, RT
Target envelope: make `signal-analysis-loudness` a stronger delivery and
runtime-diagnostics building block by moving beyond the current offline mono
surface.

## Problem

Signal already has a useful loudness meter, but important depth is still
missing:

1. multichannel handling is not yet a first-class analysis surface,
2. K-weighting and true-peak behavior need broader sample-rate and delivery
   confidence,
3. short-term/momentary traces and richer dynamics summaries are still shallow,
4. runtime and offline consumers would otherwise diverge on loudness logic.

## Goals

- deepen BS.1770-style loudness support beyond the current mono-heavy path
- improve true-peak handling across more sample rates and channel layouts
- add stronger dynamics and trace surfaces for downstream diagnostics
- keep delivery-facing outputs explicit and confidence-backed

## Non-Goals

- full mastering decision policy
- product-local loudness UI semantics
- final broadcast/podcast preset catalog

## Execution Plan

### 004.1 Multichannel and sample-rate depth

- [x] make channel weighting and aggregation an explicit public contract
- [x] broaden K-weighting and true-peak support across more sample-rate cases
- [x] preserve deterministic fallback behavior where exact parity is not yet in
      place

### 004.2 Richer dynamics surfaces

- [x] add short-term and momentary trace outputs where useful
- [x] deepen dynamics summaries beyond one integrated figure and one range
- [x] keep runtime-diagnostics reuse in scope without leaking runtime policy

### 004.3 Validation and evidence

- [x] add fixture coverage for mono, stereo, and multichannel cases
- [x] compare against reference expectations where practical
- [x] log closure evidence and remaining delivery gaps

## Acceptance Signals

1. `signal-analysis-loudness` is credible for both catalog analysis and runtime
   diagnostics reuse.
2. Channel layout and sample-rate behavior are explicit enough that callers do
   not need hidden assumptions.
3. Loudness and dynamics outputs are richer without collapsing into product
   policy.

## Risks and Mitigations

- Risk: standards work expands into endless edge-case chasing.
- Mitigation: lock one practical multichannel/dynamics slice and record the
  rest explicitly.
- Risk: runtime concerns pollute generic analysis APIs.
- Mitigation: keep outputs descriptive and host-neutral.

## Evidence Requirements

- [x] logs under `docs/logs/YYYY-MM/`
- [x] multichannel and sample-rate fixture coverage
- [x] closeout notes on any remaining standards gaps

## Current Evidence

The opening `g02.004` tranche moves `signal-analysis-loudness` past a
mono-mixdown surface and into an explicit aggregation contract:

- `signal-analysis-loudness` now exposes:
  - per-channel loudness and true-peak summaries
  - aggregation metadata for channel weighting, sample-rate support, and
    true-peak oversample factor
- the current loudness path now preserves:
  - channel-aware mono/stereo aggregation
  - deterministic counted-layout fallback for generic multichannel buffers
  - explicit distinction between native `48 kHz`, resampled-to-`48 kHz`, and
    unweighted fallback analysis
- fixture coverage now pins:
  - stereo duplicate material aggregating louder than mono under equal weights
  - deterministic generic multichannel fallback behavior
  - non-native-rate resampling and non-`48 kHz` fallback reporting

The current trace-and-dynamics tranche now makes loudness movement reusable
instead of collapsing everything into one integrated figure:

- `signal-analysis-loudness` now also exposes:
  - momentary and short-term trace surfaces with explicit window/hop metadata
  - a compact dynamics summary built from those traces
- the current loudness path now preserves:
  - trace outputs on the same aggregated multichannel/sample-rate contract
  - delivery-facing summary fields such as target offset and peak-to-loudness
    spread without adding product policy
- fixture coverage now also pins:
  - louder sections surfacing later in the momentary trace
  - dynamics summaries reacting to real level-step material instead of static
    amplitude-only checks

The closing diagnostics-and-reference tranche now freezes which loudness fields
belong to runtime monitoring instead of leaving that choice implicit:

- `signal-analysis-loudness` now also exposes:
  - a bounded runtime-diagnostics summary with current momentary and short-term
    loudness plus recent trace tails
- the current loudness path now preserves:
  - the same delivery metrics across full offline results and compact runtime
    diagnostics
  - stronger reference-style expectations for channel duplication and amplitude
    scaling rather than loose monotonic assertions
- fixture coverage now also pins:
  - stereo duplicate energy near the expected `+3.01 LU`
  - generic four-channel duplicate energy near the expected `+6.02 LU`
  - amplitude scaling near the expected `20 * log10(gain ratio)` delta

## Residual Scope

`g02.004` is complete for the current loudness target envelope.

Remaining deeper loudness work, if reopened later, should be treated as future
scope beyond this milestone:

- speaker-role-aware surround weighting rather than generic counted-layout
  fallback
- broader sample-rate-specific K-weighting parity beyond the current `48 kHz`
  anchor
- richer standards or corpus comparisons against external reference tools

## Next Task

Open `g02.005` by turning `signal-analysis-character` into a real descriptor-pack
surface: add practical spectral descriptors, freeze their reduction policy, and
group them into reusable packs before deepening transient-shape work.
