# 001 - Audio Precision, Sample Format, And File Bit-Depth Policy

Status: active
Owner: core-product
Updated: 2026-03-13
Applies to: `signal-primitives`, `signal-runtime`, `signal-plugin-*`, `signal-hardware-*`

## Purpose

State one durable policy for internal audio precision, file-format precision,
hardware sample-format negotiation, and future plugin double-precision support
so those decisions do not drift milestone by milestone.

## Current Runtime Truth

- Signal's shared internal sample type is `f32`.
- Runtime audio buffers are interleaved `f32`.
- Runtime media decode normalizes supported source formats into `f32`.
- Offline render and recording capture currently emit 32-bit float WAV.
- Signal already uses `f64` selectively for timing, ratio math, accumulators,
  and plugin-control translation where that improves correctness without making
  the whole audio path double precision.

## Policy

### Internal Processing Precision

- Keep the shared engine and graph processing path `f32` by default.
- Treat any future engine-wide `f64` path as an explicit exceptional design
  choice that needs profiling, memory-bandwidth analysis, and downstream
  compatibility justification.
- Allow targeted `f64` usage for:
  - tempo and timeline math
  - warp or resample ratios and source-position indexing
  - loudness, RMS, and other long-window accumulators
  - plugin parameter, note-expression, or host-control APIs that are naturally
    double precision

### File And Artifact Precision

- Keep 32-bit float WAV as the default high-fidelity Signal-owned render,
  freeze, capture, and cache artifact format.
- Treat integer PCM import and export breadth as an interchange concern, not as
  evidence that the engine itself should stop being `f32`.
- Support import of common integer PCM material where practical, including
  16-bit, 24-bit, and 32-bit integer PCM, by normalizing into `f32`.
- Plan integer PCM export as a later format-policy layer above the runtime
  engine, with explicit dither or quantization policy when that surface opens.

### Hardware Sample Formats

- Keep hardware sample format as a negotiated edge contract.
- Normalize negotiated device I/O into Signal's internal `f32` engine path
  unless a backend-specific reason justifies a different boundary adapter.
- Do not treat support for `I16` or `I32` hardware endpoints as a reason to
  widen the internal graph sample type.

### Plugin Precision

- Treat plugin precision separately from engine precision.
- If later adapters expose plugins that can or must process at double
  precision, prefer explicit adapter capability negotiation plus narrow
  conversion boundaries over a global engine-wide type change.
- Keep runtime receipts explicit about any adapter-private precision downgrade
  or unsupported double-precision path once VST3 or AU depth lands.

## What This Policy Does Not Claim

- no commitment to a full `f64` mix engine
- no commitment yet to 64-bit float file interchange as a product-facing
  default
- no claim that integer PCM export policy, dithering policy, or archival media
  policy is already complete

## Practical Guidance For Loophole And Other Consumers

- If a consumer asks “does Signal support 32-bit audio?”, the default reading
  should be:
  - yes for internal `f32` processing
  - yes for 32-bit float render and capture artifacts
  - partially for integer PCM interoperability, depending on the specific file
    or device edge
- If a consumer asks for higher precision, first distinguish:
  - internal engine precision
  - plugin API precision
  - file import/export bit depth
  - hardware endpoint sample format

## Next Task

Freeze the first explicit import/export precision matrix once Signal opens the
next media-interchange or adapter-capability tranche, so integer PCM export and
future plugin double-precision negotiation inherit one stable policy.
