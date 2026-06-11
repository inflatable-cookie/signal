# 018 - Disk Streaming Clip Sources

Status: planned
Owner: core-product
Created: 2026-06-11
Depends on: g10.011
Vision tags: `STREAMING`, `MEMORY`

## Problem

Clips are whole-file decodes into Arc buffers, and RenderSampleCache
decodes synchronously inside plan compile — a UI stall the moment files are
minutes long, untenable once recording lands. Long media needs a streaming
source the executor can drain without allocation.

## Goals

- [ ] RenderSource::Stream: per-clip lock-free SPSC ring drained by the executor; underrun policy = silence + atomic counter (never block)
- [ ] read-ahead thread: seek-aware (primes on transport seek), decodes via symphonia ahead of the playhead
- [ ] ring handles are per-node state moving across plan swaps via g10.011's handoff map
- [ ] async decode for the sample cache: imports decode off the control thread; compile consumes ready buffers or registers pending decodes
- [ ] policy: short files stay in-memory Samples; threshold configurable
- [ ] peak/waveform cache responsibility noted for pulse (Signal exposes decode taps)

## Execution Plan

### Batch 18.1 - Stream Source

- [ ] ring + executor drain + underrun counters; soak with streaming clips

### Batch 18.2 - Read-Ahead And Async Decode

- [ ] seek-aware prefetch thread; async import decode; threshold policy

## Acceptance Criteria

- [ ] hour-long file plays with bounded memory and zero callback allocation
- [ ] seek mid-file resumes within one read-ahead window without underrun
- [ ] import of a long file no longer stalls the control thread

## Next Task

Anticipative rendering (prework concept) re-derives on this substrate later.
