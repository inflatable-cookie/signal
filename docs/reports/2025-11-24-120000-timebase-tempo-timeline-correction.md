# Timebase, Tempo & Timeline Accuracy Implementation Report

**Date:** 2025-11-24  
**Implementation:** Unified timebase system with real tempo, time signature, sample rate, and computed timeline end

---

## Overview

This report documents the implementation of a unified timebase system across Pulse, Signal, and Aura. The changes eliminate hardcoded assumptions about tempo (120 BPM), time signature (4/4), timeline length (128 beats), and sample rate (44100 Hz), replacing them with authoritative values derived from session state and engine configuration.

---

## 1. Unified Timebase Module in Pulse

**Location:** `pulse/src/util/time_conversion.rs`

Expanded the existing time conversion utilities to include all necessary conversion functions:

- `beats_to_seconds(beats, tempo)` - Convert musical time to wall-clock time
- `seconds_to_beats(seconds, tempo)` - Convert wall-clock time to musical time
- `beats_to_samples(beats, tempo, sample_rate)` - Convert musical time to sample time (existing)
- `samples_to_beats(samples, tempo, sample_rate)` - Convert sample time to musical time (existing)
- `bars_beats_from_beats(beats, time_signature)` - Extract bar and beat from beat position

All functions are pure and do not depend on global state. They use real tempo, time signature, and sample rate values passed as parameters.

**Tests:** Added comprehensive unit tests for all new conversion functions, including edge cases and round-trip verification.

---

## 2. Sample Rate Handshake Between Signal and Pulse

**Signal Changes:**
- **`signal/src/ipc/DomainDispatcher.cpp`**: Updated `engine.state` event to include `sampleRate` and `blockSize` in the payload
- Signal now sends its actual sample rate (44100 Hz) and block size (512) to Pulse when reporting engine state

**Pulse Changes:**
- **`pulse/src/domains/automation_domain.rs`**: Already retrieves sample rate from `session.engine.diagnostics.sample_rate` with a fallback to 44100.0
- Sample rate is stored in `EngineState.diagnostics` when Signal sends diagnostics events
- All time conversions now use the real sample rate from engine diagnostics

---

## 3. Timeline End Calculation

**Location:** `pulse/src/model/session_timeline_builder.rs`

Updated `calculate_timeline_end()` to:

- Compute timeline end from actual session content:
  - Rightmost clip end position across all tracks
  - Rightmost marker position
  - Loop region end (if enabled and greater than content end)
- Use a minimum of **4 bars (16 beats)** instead of 32 bars (128 beats) for empty sessions
- Add 4 beats of padding to ensure content is fully visible

**Changes:**
- Replaced hardcoded `128.0` fallback with `16.0` (4 bars at 4/4 time)
- Timeline end is now computed dynamically based on actual session content

---

## 4. Real Timebase in Project Snapshots

**Location:** `pulse/src/domains/project_domain.rs`

Updated `build_project_snapshot()` to:

- Accept `SessionState` as a parameter to access transport tempo and timeline snapshot
- Use real tempo and time signature from `SessionTimelineSnapshot` instead of hardcoded defaults
- All project snapshot payloads now include accurate timebase information

**Changes:**
- Modified `build_project_snapshot(project, session)` signature
- Updated all call sites to pass session state
- Project snapshots now reflect actual session tempo and time signature

---

## 5. Signal Transport and Engine Updates

**Signal Changes:**

- **`signal/src/core/TransportState.hpp`**: Added `tempo` field (defaults to 120.0 BPM)
- **`signal/src/domains/TransportDomain.cpp`**:
  - Updated all time conversions to use real sample rate from `EngineHost::getSampleRate()`
  - Updated beats-to-seconds conversions to use real tempo from `transport.tempo`
  - Added `setTempo` command handler to update transport tempo
- **`signal/src/domains/EngineDomain.cpp`**: Updated `scheduleSession` to use real tempo from transport state
- **`signal/src/ipc/DomainDispatcher.cpp`**:
  - Updated `engine.state` event to include `sampleRate` and `blockSize`
  - Updated `transport.state` event to use real tempo for beats conversions

**Removed Hardcoded Values:**
- Replaced all instances of `44100.0` with `_engineHost->getSampleRate()`
- Replaced all instances of `120.0` tempo assumptions with `transport.tempo`

---

## 6. Aura UI Component Updates

**Location:** `aura/src/renderer/ui/timeline/`

Updated timeline components to use real timebase values:

- **`TimelineScrubber.svelte`**: Changed fallback from `128` beats to `16` beats (4 bars)
- **`TrackLane.svelte`**: Changed fallback from `128` beats to `16` beats

**Note:** Aura already uses tempo and time signature from `SessionTimelineSnapshot` for formatting. The main change was updating the timeline end fallback to match the new minimum (16 beats instead of 128).

---

## 7. Time Signature Handling

**Current State:**
- Time signature is stored in `SessionTimelineSnapshot` and propagated to Aura
- Default time signature is `[4, 4]` when no project is loaded
- Time signature is included in project snapshots and timeline snapshots

**Future Work:**
- Full time signature map support (time signature changes over time)
- Time signature changes in transport domain
- Bars/beats calculations using real time signature

---

## 8. Known Limitations and Follow-Ups

### Multi-Tempo Map
- Currently supports single global tempo
- Future: Tempo map with tempo changes over time
- Future: Tempo automation and tempo curves

### Time Signature Map
- Currently supports single global time signature
- Future: Time signature changes over time
- Future: Complex time signatures (e.g., 7/8, 5/4)

### Sample Rate Negotiation
- Signal currently uses fixed 44100 Hz sample rate
- Future: Dynamic sample rate negotiation based on audio device
- Future: Sample rate changes during runtime

### Timeline End Calculation
- Currently considers clips, markers, and loop region
- Future: Consider automation curve end points
- Future: Consider plugin automation end points

---

## 9. Testing

**Pulse Tests:**
- Unit tests for all timebase conversion functions in `pulse/src/util/time_conversion.rs`
- Tests verify:
  - Correct beats ↔ seconds conversions at various tempos
  - Correct beats ↔ samples conversions at various tempos and sample rates
  - Bar/beat extraction from beats with different time signatures
  - Round-trip conversions maintain accuracy

**Signal Tests:**
- Manual verification that sample rate is sent in engine.state events
- Manual verification that tempo is used correctly in transport conversions

**Aura Tests:**
- UI components use real timebase values from snapshots
- Timeline scaling uses computed timeline end

---

## 10. Summary of Changes

### Files Modified

**Pulse:**
- `src/util/time_conversion.rs` - Expanded with new conversion functions
- `src/model/session_timeline_builder.rs` - Updated timeline end calculation
- `src/persistence/session_builder.rs` - Use real time signature from timeline snapshot
- `src/domains/project_domain.rs` - Use real tempo/time signature in project snapshots

**Signal:**
- `src/core/TransportState.hpp` - Added tempo field
- `src/domains/TransportDomain.cpp` - Use real sample rate and tempo
- `src/domains/EngineDomain.cpp` - Use real tempo from transport
- `src/ipc/DomainDispatcher.cpp` - Include sample rate in engine.state, use real tempo in transport.state

**Aura:**
- `src/renderer/ui/timeline/TimelineScrubber.svelte` - Updated fallback to 16 beats
- `src/renderer/ui/timeline/TrackLane.svelte` - Updated fallback to 16 beats

### Hardcoded Values Removed

- `120.0` BPM tempo assumptions → Real tempo from transport/session
- `128.0` beats timeline end → Computed from session content (minimum 16 beats)
- `44100.0` Hz sample rate assumptions → Real sample rate from engine diagnostics
- `[4, 4]` time signature hardcoding → Real time signature from timeline snapshot (still defaults to 4/4 for empty sessions)

---

## 11. Migration Notes

**For Developers:**
- All time conversions should use functions from `pulse/src/util/time_conversion.rs`
- Never hardcode tempo, sample rate, or time signature values
- Always retrieve tempo from `session.transport.tempo` or `SessionTimelineSnapshot`
- Always retrieve sample rate from `session.engine.diagnostics.sample_rate` with fallback
- Timeline end should be computed from session content, not hardcoded

**For Testing:**
- Test with various tempos (60, 120, 140, 180 BPM)
- Test with various sample rates (44100, 48000, 96000 Hz)
- Test with different time signatures (3/4, 4/4, 7/8)
- Verify timeline end calculation with mixed content (clips, markers, automation)

---

This implementation establishes a solid foundation for accurate timebase handling across the Loophole stack, with clear paths for future enhancements like multi-tempo maps and time signature changes.

