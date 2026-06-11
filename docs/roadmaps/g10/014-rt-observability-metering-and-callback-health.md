# 014 - RT Observability Metering And Callback Health

Status: planned
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

- [ ] per-node atomic peak/RMS published from render_block (seqlock or atomic pair per node, table sized at compile)
- [ ] callback interval/duration counters; xrun inference (missed deadline = interval > buffer duration + margin) surfaced as counters
- [ ] cpal error detail captured into shared state instead of dropped
- [ ] host plumbing: Aura polls meters at UI rate; console meters move
- [ ] fake clocked backend behind OutputStreamBackend for device-less CI soak (trait boundary makes this nearly free)

## Execution Plan

### Batch 14.1 - Meter Taps

- [ ] atomic meter table + executor publication; Aura console meters live

### Batch 14.2 - Health

- [ ] callback timing counters, xrun inference, error capture
- [ ] fake clocked backend + CI soak run

## Acceptance Criteria

- [ ] meters move in Aura's console during playback
- [ ] synthetic starvation registers as xrun counters
- [ ] CI runs a clocked soak without hardware

## Next Task

g10.016 (output-time honesty) builds on the health instrumentation.
