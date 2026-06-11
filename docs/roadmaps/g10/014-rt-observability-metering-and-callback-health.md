# 014 - RT Observability Metering And Callback Health

Status: done
Owner: core-product
Created: 2026-06-11
Depends on: g10.010
Vision tags: `OBSERVABILITY`, `RT-SAFETY`

## Problem

SharedState exposes position/playing/parked — nothing else. No meters for
the UI, no callback-health signal, no xrun inference, and the cpal error
callback throws away the error detail. You cannot harden what you cannot
see, and Aura's console renders meters that show nothing.

## Goals

- [x] per-node atomic peak/RMS published from render_block (fixed 256-slot atomic-pair table in SharedState, generation-stamped per plan install)
- [x] callback interval/duration counters; xrun inference (missed deadline = interval > 1.5 × block duration at the plan rate) surfaced as counters
- [x] cpal error detail captured into shared state instead of dropped (`OutputStreamHandle::last_error`, Display string behind a mutex written on cpal's backend thread)
- [x] host plumbing: Aura polls meters at UI rate; console meters move
- [x] fake clocked backend behind OutputStreamBackend for device-less CI soak (trait boundary makes this nearly free)

## Execution Plan

### Batch 14.1 - Meter Taps

- [x] atomic meter table + executor publication; Aura console meters live

### Batch 14.2 - Health

- [x] callback timing counters, xrun inference, error capture
- [x] fake clocked backend + CI soak run

## Acceptance Criteria

- [x] meters move in Aura's console during playback (live engine meters override pulse-model metering, exponential UI decay between polls)
- [x] synthetic starvation registers as xrun counters (`tests/fake_clocked_soak.rs`)
- [x] CI runs a clocked soak without hardware (workspace test run covers it)

## Progress (2026-06-11)

- SharedState: 256-slot atomic meter table (peak+RMS f32 bits per topo
  stage, generation-stamped so readers never mislabel slots across plan
  swaps), callback_count, last/max callback duration micros, xrun_count
  (interval > 1.5× block duration). `RenderPlaneController::meters()`
  resolves slots to stage ids via the last topology; mismatched generation
  returns empty for at most one block. Meters publish post-fader from each
  stage's final scratch; silence publishes zeros. All Relaxed; tearing
  documented as cosmetic.
- cpal error detail captured (backend worker thread — allocation safe
  there, documented) and surfaced via `OutputStreamHandle::last_error()`.
- `FakeClockedBackend` in signal-hardware: spec-exact ticking thread for
  device-less CI soak; new render-plane integration test starves every
  32nd callback and asserts xrun inference, meter movement, and health
  counters — runs in the normal workspace suite.
- Aura: playback status carries xruns + callback timing; new
  get_aura_stage_meters command; renderer maps chains→stages via a
  BigInt FNV-1a-64 parity-tested against Rust reference hashes; console
  meters poll at 50 ms with exponential decay between polls and fall back
  to model metering when no engine data.

## Next Task

g10.016 (output-time honesty) builds on the health instrumentation.
