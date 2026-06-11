# 019 - Transport Regions Loop Click Count-In

Status: complete
Owner: core-product
Created: 2026-06-11
Depends on: g10.012
Vision tags: `TRANSPORT`, `PRODUCT`

## Problem

The executor loops clip sources, not transport regions: pulse's
PulseLoopRegion reaches no audio, the metronome does not exist, and
recording (g10.017) wants a count-in. Loop-region wrap must happen in the
executor — a control-side seek would jitter by a mailbox round-trip.

## Goals

- [ ] loop region in the executor: wrap position_frames at loop_end with the existing edge-ramp declick; region set via plan or transport command
- [ ] metronome: pulse-compiled click clips from the tempo map on a dedicated always-on lane (no engine change beyond the lane)
- [ ] count-in: pre-roll offset on the play edge (transport command), click audible during pre-roll
- [ ] pulse/Aura wiring: loop braces audible, click toggle, count-in setting

## Execution Plan

### Batch 19.1 - Loop Region

- [ ] executor wrap + declick + command plumbing

### Batch 19.2 - Click And Count-In

- [ ] tempo-map click compilation; pre-roll; product wiring

## Acceptance Criteria

- [ ] loop region cycles sample-accurately without click
- [ ] metronome lands on tempo-map beats
- [ ] count-in precedes recording start by the configured bars

## Progress (2026-06-11)

- Loop region in the executor: SetLoopRegion command (typed error on
  inverted bounds); blocks crossing loop_end render two segments into one
  buffer (block-level gain/automation/meter/limiter math untouched) with a
  64-frame micro-fade around the wrap — active only on wrap blocks, golden
  hash unchanged. Clock lands at loop_start + remainder. Two stream-source
  fixes the loop tests forced: held-chunk retention capped at loop end
  while inside a region, and a furthest-ahead eviction when a backward
  jump finds all slots full.
- Metronome compiled by pulse (signal never sees BPM): reserved
  "metronome" stage seed, per-beat TestTone clips (50 ms, 1760/1320 Hz
  accents, 4/4) from the transport tempo across the extent plus one loop
  pass; SetMetronomeEnabled command + transport snapshot exposure.
- Count-in: CaptureSession::start_with_skip discards the first N ring
  frames before writing (rescaled to the negotiated rate);
  aura_start_recording(count_in_bars) seeks to anchor − bars·beat·4,
  starts transport, and skips exactly the pre-roll in the take — placement
  anchor and latency compensation unchanged. Metronome not auto-enabled.
- Aura: loop state pushed through sync_transport (re-primed after stream
  rebuilds); timer count-in toggle + music metronome toggle in the
  timeline toolbar.
- Owed: manual audition (loop a clip, click on, 1-bar count-in record).

## Next Task

Recording usability complete; MIDI remains the next instrument-side unlock (backlog).
