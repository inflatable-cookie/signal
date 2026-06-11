# 018 - Disk Streaming Clip Sources

Status: complete
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

## Progress (2026-06-11)

- `RenderSource::Stream(RenderStreamHandle)`: Arc-shared pointer-equal
  handles created once per streamed asset; chunk-mailbox design mirrors the
  proven plan-swap pattern (StreamChunk { start_frame, Arc frames };
  bounded lock-free Vyukov rings replace sync_channel because handles live
  in clonable specs — same semantics, alloc-free try_push/try_pop; retired
  chunks always return control-side, retire-full parks in-slot). Executor
  holds 4 chunks per stream clip, publishes wanted_frame per block, retires
  behind-playhead and out-of-lookahead chunks; missing frame = silence +
  underrun counter. Seeks fall out of the wanted-frame jump. Held chunks
  MOVE across plan swaps via the g10.011 clip maps (inherit_state now takes
  &mut previous). Shared interpolate_source_frame helper unifies
  Samples/Stream sampling — golden render hash unchanged.
- Pulse: decode_wav_window + probe_wav (header-only); RenderSampleCache
  resolves assets past the 30 s threshold to Streamed (feeders +
  decode cursors behind an Arc<Mutex> leaf); service_streams pumps all
  feeders one pass (retire, seek-detect, decode 32k-frame chunks to a
  4-chunk lookahead). Bounce compiles with streaming disabled — offline
  renders faster than realtime with no feeder, documented.
- Aura: one aura-stream-feeder thread (50 ms cadence) pumping the cache's
  streaming worker; streaming mutex is a leaf in the documented lock order
  so disk decode never blocks commands.
- Engine tests: 1:1 sample accuracy, underrun count + clean resume, far
  seek retire/refill, 44.1k→48k streamed polyphase >60 dB SNR, held-chunk
  continuity across identity recompiles, pointer equality. Pulse: threshold
  resolve, recompile pointer-equal, worker chunk flow, windowed-vs-full
  decode equivalence. Soak still zero-alloc.
- Known limitation: simultaneously audible clips of the SAME streamed asset
  share one mailbox/hint — per-(asset,clip) handles are the named follow-up
  if overlapping placements arrive. Live end-to-end aura playback of a
  >30 s wav is the owed operator proof.

## Next Task

Anticipative rendering (prework concept) re-derives on this substrate later.
