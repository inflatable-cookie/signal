# Signal Engine Foundations (A1–A3) – Implementation Report

**Date:** 2025-11-24  
**Scope:** EngineHost + AudioBackend interfaces and basic wiring

## Summary

Implemented the foundational interfaces and wiring for the Signal audio engine, establishing the `EngineHost::renderBlock` entry point and a backend-agnostic `AudioBackend` abstraction. The implementation includes a placeholder `MiniaudioBackend` that correctly calls `EngineHost::renderBlock` from the audio callback.

## Changes Made

### 1. Core Types

#### EngineRenderContext (`src/core/EngineRenderContext.hpp`)
- Created struct containing:
  - `hostTimeSeconds` (double) – monotonic host time from backend
  - `sampleRate` (double) – current sample rate
  - `blockSize` (int) – number of frames in block
  - `playheadSamples` (uint64_t) – transport position (informational, updated by EngineHost)

#### AudioBus (`src/core/AudioBus.hpp`)
- Created abstraction for multi-channel audio buffers
- Supports interleaved audio format (channels × frames)
- Provides read-only access for input buses, read-write for output buses
- Methods:
  - `numChannels()`, `numFrames()`, `totalSamples()`
  - `data()` – raw pointer access (const and non-const)
  - `sample(frame, channel)` – read sample
  - `setSample(frame, channel, value)` – write sample
  - `clear()` – zero all samples
  - `isReadOnly()` – check access mode

### 2. EngineHost Interface

#### Added `renderBlock` Method
- Public method: `void renderBlock(EngineRenderContext& ctx, AudioBus& input, AudioBus& output)`
- Real-time safe: no allocations, locks, or I/O
- Current implementation: produces silence (test tone code commented out for future use)
- Includes TODO comments for future phases:
  - Phase B: Schedule → clips
  - Phase C: Mixer gain/mute/solo
  - Phase D: Automation (volume & pan)
  - Phase E: Loop handling and metering

### 3. AudioBackend Interface

#### AudioBackendConfig (`src/backend/AudioBackendConfig.hpp`)
- Configuration struct for backend initialization:
  - Device selection (optional input/output device IDs)
  - Preferred sample rate and buffer size
  - Number of input/output channels

#### AudioBackend (`src/backend/AudioBackend.hpp`)
- Pure virtual interface for backend-agnostic audio I/O
- Methods:
  - `initialise(config)` – setup backend
  - `shutdown()` – cleanup
  - `start()` / `stop()` – control streaming
  - `setRenderCallback(callback)` – register engine callback
  - `getSampleRate()`, `getBufferSize()`, `getNumInputChannels()`, `getNumOutputChannels()` – query current state

### 4. MiniaudioBackend Implementation

#### Placeholder Implementation (`src/backend/MiniaudioBackend.*`)
- Simulates audio callbacks using a dedicated thread
- Maintains host time tracking (monotonic, in seconds)
- Creates `EngineRenderContext` and `AudioBus` objects for each callback
- Calls registered render callback from audio thread
- Real-time safe: no allocations or locks in callback path

**Note:** This is a placeholder that will be replaced with actual miniaudio integration in a future phase. The interface and wiring are correct; only the underlying device I/O needs to be implemented.

### 5. EngineHost Integration

- Updated `EngineHost` to own an `AudioBackend` instance (in addition to legacy `AudioThread` for backward compatibility)
- Added `setupAudioBackend()` method that:
  - Creates `MiniaudioBackend`
  - Configures with default settings (44.1kHz, 512 samples, stereo output)
  - Sets render callback to `EngineHost::renderBlock`
- Modified `start()` / `stop()` to use `AudioBackend` when available
- Updated `getSampleRate()` / `getBlockSize()` to query backend

### 6. Build System

- Updated `src/CMakeLists.txt` to include `backend/MiniaudioBackend.cpp`
- Added `test_audio_bus.cpp` to test suite
- Updated `tests/CMakeLists.txt` to include new test file

### 7. Tests

- Created comprehensive `AudioBus` tests:
  - Basic properties (channels, frames, samples)
  - Read-only vs writable access
  - Sample read/write operations
  - Out-of-bounds access handling
  - Clear operation
- All tests pass (16 test cases, 76 assertions)

## Deviations from Spec

1. **EngineRenderContext.playheadSamples**: Added this field to provide transport position information. The spec allows for additional fields, and this is informational only (updated by EngineHost).

2. **AudioBackend query methods**: Added `getSampleRate()`, `getBufferSize()`, `getNumInputChannels()`, `getNumOutputChannels()` for convenience. These don't conflict with the spec and are useful for engine state queries.

3. **Legacy AudioThread support**: Kept `AudioThread` alongside `AudioBackend` for backward compatibility. This will be removed in a future refactor once all code paths use the backend.

## Build and Test Results

### Signal
- **Build:** ✅ Success
  ```bash
  cmake -S . -B build
  cmake --build build
  ```
- **Tests:** ✅ All pass (16 test cases, 76 assertions)
  ```bash
  ctest --test-dir build
  ```

### Pulse
- **Build:** ✅ Success
  ```bash
  cargo build
  ```
- **Tests:** ✅ All pass (117 unit tests + 4 integration tests)
  ```bash
  cargo test
  ```

## Known TODOs for Future Phases

- **Phase B:** Implement schedule → clips processing in `renderBlock`
- **Phase C:** Implement mixer gain/mute/solo in `renderBlock`
- **Phase D:** Implement automation (volume & pan) in `renderBlock`
- **Phase E:** Implement loop handling and metering in `renderBlock`
- **Future:** Replace placeholder `MiniaudioBackend` with actual miniaudio integration
- **Future:** Remove legacy `AudioThread` once all code paths use `AudioBackend`

## Files Created

- `src/core/EngineRenderContext.hpp`
- `src/core/AudioBus.hpp`
- `src/backend/AudioBackendConfig.hpp`
- `src/backend/AudioBackend.hpp`
- `src/backend/MiniaudioBackend.hpp`
- `src/backend/MiniaudioBackend.cpp`
- `tests/test_audio_bus.cpp`

## Files Modified

- `src/core/EngineHost.hpp` – added `renderBlock` method, backend support
- `src/core/EngineHost.cpp` – implemented `renderBlock`, integrated `AudioBackend`
- `src/CMakeLists.txt` – added backend sources
- `tests/CMakeLists.txt` – added audio bus tests

## Alignment with Spec

The implementation aligns with `docs/specs/audio-backend-plugin-hosting-spec-outline.md`:
- `EngineRenderContext` matches spec (with added `playheadSamples` field)
- `AudioBackend` interface matches spec (with added query methods)
- `MiniaudioBackend` follows spec structure (placeholder implementation)
- `EngineHost::renderBlock` matches spec signature and real-time safety requirements

