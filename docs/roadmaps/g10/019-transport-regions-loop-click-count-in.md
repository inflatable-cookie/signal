# 019 - Transport Regions Loop Click Count-In

Status: planned
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

## Next Task

Recording usability complete; MIDI remains the next instrument-side unlock (backlog).
