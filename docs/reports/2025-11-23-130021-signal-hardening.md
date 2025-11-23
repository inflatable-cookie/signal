# Signal Skeleton Hardening Report

**Date:** 2025-11-23 13:00:21 UTC  
**Task:** Harden Signal skeleton (engine, transport, concurrency)

---

## Summary

This report documents the hardening of the Signal skeleton implementation to ensure:

- Explicit and documented concurrency model
- Correct participation in engine and transport domains
- State and diagnostics events sent back to Pulse
- Safe audio thread lifecycle (even though currently just produces silence/test tone)
- Graceful shutdown coordinated with Pulse

---

## Files Modified

### Core Components

- `src/core/SignalApp.hpp` - Added file-level comments describing thread model
- `src/core/SignalApp.cpp` - Added diagnostics timer and improved shutdown handling
- `src/core/EngineHost.hpp` - Extended with proper lifecycle states (Stopped, Starting, Running, Error), error handling, and transport state
- `src/core/EngineHost.cpp` - Implemented full lifecycle management, error handling, transport state access, and diagnostic stubs
- `src/core/AudioThread.hpp` - Added file-level comments describing real-time constraints
- `src/core/TransportState.hpp` - **New file**: Defines transport state structure

### IPC Components

- `src/ipc/TcpServer.hpp` - Added `broadcastDiagnostics()` template method
- `src/ipc/TcpServer.cpp` - Added file-level comments describing thread model
- `src/ipc/DomainDispatcher.cpp` - Enhanced engine and transport state event emission, added heartbeat handling

### Domain Handlers

- `src/domains/EngineDomain.cpp` - Added shutdown command handling
- `src/domains/TransportDomain.cpp` - **Major update**: Implemented full transport command handling (play, stop, seek, setLoopEnabled, setLoopRegion)

### Documentation

- `docs/decisions/2025-11-23-signal-threading-and-concurrency.md` - **New file**: ADR documenting the concurrency model

---

## Concurrency Model

### Thread Architecture

Signal uses a **three-thread architecture**:

1. **Main Thread**
   - Process entry point (`main()`)
   - Owns `SignalApp`, `EngineHost`, `IpcRouter`
   - Sets up and starts IPC server
   - Coordinates shutdown

2. **IPC Thread (Asio Event Loop)**
   - Runs in Asio `io_context` worker threads
   - Handles TCP connection accept and client sessions
   - Reads envelopes from Pulse
   - Parses JSON envelopes
   - Dispatches commands to domain handlers synchronously
   - Domain handlers update `EngineHost`/`TransportState` directly
   - **Does NOT** mutate state from multiple threads concurrently (handlers run synchronously)

3. **Audio Thread**
   - Dedicated thread for audio processing
   - High priority (platform-specific, to be configured)
   - Processes audio buffers in real-time
   - Reads engine/transport state via lock-free atomics
   - **Never** blocks, allocates, or performs I/O

### Communication Patterns

- **IPC Thread → Engine/Transport State**: Direct synchronous updates via domain handlers (no concurrent access)
- **Main Thread → Audio Thread**: Atomic flags (`std::atomic<bool>`) for state changes
- **Audio Thread → Main Thread**: Minimal stats via atomics (CPU load, XRuns - currently stubbed)

### State Ownership

- `EngineHost`: Owned by main thread, accessed from IPC thread handlers (synchronously)
- `TransportState`: Owned by `EngineHost`, accessed from IPC thread handlers (synchronously)
- Audio thread reads state via atomic flags and lock-free snapshots

See `docs/decisions/2025-11-23-signal-threading-and-concurrency.md` for full details.

---

## Engine Lifecycle

### States

The `EngineHost` now supports four lifecycle states:

- `Stopped` - Engine is stopped, audio thread is not running
- `Starting` - Transition state when starting (set before audio thread starts)
- `Running` - Engine is running, audio thread is active
- `Error` - Engine encountered an error (includes error message)

### Commands Handled

- `engine.start` - Starts the engine (idempotent if already running, fails if in error state)
- `engine.stop` - Stops the engine
- `engine.reset` - Stops engine and clears error state
- `engine.shutdown` - Initiates graceful shutdown
- `engine.heartbeat` - Responds with current engine state

### State Events

After processing engine commands, Signal emits `engine.state` events with:

- `lifecycle`: One of "stopped", "starting", "running", "error"
- `lastError`: Error message string or `null` if no error

Correlation IDs are set to the triggering command's ID for command responses.

---

## Transport Domain

### TransportState Structure

Defined in `src/core/TransportState.hpp`:

```cpp
struct TransportState {
    bool isPlaying;
    double positionSeconds;
    bool loopEnabled;
    std::optional<LoopRegion> loopRegion;
};
```

### Commands Handled

- `transport.play` - Sets `isPlaying = true`
- `transport.stop` - Sets `isPlaying = false`
- `transport.seek` - Updates position (accepts `seconds` or `positionBeats` in payload)
- `transport.setLoopEnabled` - Sets loop enabled flag
- `transport.setLoopRegion` - Sets loop region (accepts seconds or beats-based region)

### State Events

After processing transport commands, Signal emits `transport.state` events with:

- `isPlaying`: Boolean
- `positionBeats`: Position in beats (currently converted from seconds using 120 BPM assumption)
- `loopEnabled`: Boolean
- `loopRegion`: Object with `startBeats` and `endBeats`, or `null`

Correlation IDs are set to the triggering command's ID.

**Note**: Full beats-based timing support is not yet implemented. Currently assumes 120 BPM for conversions.

---

## Diagnostics Events

### Periodic Emissions

Signal emits `engine.diagnostics` events **every second** via an Asio steady timer:

- `cpuLoad`: Double (currently stubbed to 0.0)
- `xruns`: Integer (currently stubbed to 0)
- `engineState`: String lifecycle state
- `sampleRate`: Double (44100.0)
- `blockSize`: Integer (512)
- `transportState`: String ("playing" or "stopped")

These events have `correlationId: null` since they are unsolicited.

### Implementation

The diagnostics timer is set up in `SignalApp::run()` and broadcasts to all connected clients via `TcpServer::broadcastDiagnostics()`.

---

## Shutdown Handling

### Graceful Shutdown Flow

1. **SIGINT/SIGTERM Handling**: Signal handler sets shutdown flag, calls `EngineHost::shutdown()`, stops IPC server, and stops `io_context`
2. **engine.shutdown Command**: Handled by `EngineDomain`, which calls `EngineHost::shutdown()`
   - Sets `_shuttingDown` flag
   - Stops audio thread
   - Emits final `engine.state` event with `lifecycle: "stopped"`
   - Process continues to run (Pulse or OS signal must stop the process)

### Shutdown Sequence

1. Command received on IPC thread (or signal received on main thread)
2. Main thread processes shutdown command (sets flag, stops audio thread)
3. Audio thread observes flag and exits gracefully on next buffer
4. IPC server stops accepting new connections
5. Existing connections are closed
6. `io_context` stops
7. Process exits

**Note**: Currently, `engine.shutdown` command does not stop the `io_context` or exit the process. This must be done by Pulse (via process termination) or via SIGINT/SIGTERM. Future improvement: add a shutdown coordination mechanism.

---

## IPC Envelope Consistency

All events emitted by Signal conform to the Chorus IPC envelope specification:

- `domain`: Matches command domain ("engine", "transport")
- `name`: Event name ("state", "diagnostics", "heartbeat")
- `kind`: Always "event" for state/diagnostics
- `origin`: "signal"
- `target`: Derived from incoming command's origin (typically "pulse")
- `correlationId`: Set to command ID for command responses, `null` for unsolicited events

---

## Remaining Limitations / TODOs

1. **Diagnostics Values Stubbed**: CPU load and XRuns are currently hardcoded to 0.0 and 0 respectively. Real implementations will need to:
   - Track audio thread CPU usage
   - Count buffer underruns/overruns

2. **Beats-Based Timing**: Transport position conversions assume 120 BPM. Full musical time support (tempo, time signature, etc.) is not yet implemented.

3. **engine.shutdown Process Exit**: The `engine.shutdown` command currently only stops the engine but does not exit the process. Process exit must be handled by Pulse (process termination) or OS signals.

4. **Multi-Client Support**: Current implementation handles one client at a time. Multiple client support would require:
   - Broadcast channels for events
   - Session management per client
   - Client-specific state snapshots

5. **Audio Thread Priority**: Audio thread priority should be set to maximum on the target platform (platform-specific code needed).

6. **Error Recovery**: Error state handling is basic. Future improvements:
   - Automatic recovery from certain errors
   - Error classification (recoverable vs. fatal)
   - Detailed error reporting

---

## Testing Recommendations

- Test engine lifecycle transitions (stopped → starting → running → stopped)
- Test transport commands and state events
- Verify diagnostics events are emitted every second
- Test graceful shutdown via SIGINT/SIGTERM
- Test graceful shutdown via `engine.shutdown` command
- Verify correlation IDs are correctly propagated
- Test error state handling

---

## Conclusion

The Signal skeleton has been hardened with:

- ✅ Explicit concurrency model documented in ADR
- ✅ Proper engine lifecycle states (Stopped, Starting, Running, Error)
- ✅ Full transport domain command handling
- ✅ State events emitted after command processing
- ✅ Periodic diagnostics events
- ✅ Graceful shutdown handling (SIGINT/SIGTERM and engine.shutdown command)
- ✅ File-level comments describing thread ownership and communication patterns

The implementation is now ready for further development with real audio processing, accurate diagnostics, and full musical time support.

