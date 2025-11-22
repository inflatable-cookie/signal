# Signal IPC TCP Server Implementation Report

**Date:** 2025-11-22  
**Task:** Implement Signal IPC TCP server for JSON-line envelope communication

## Summary

Implemented a complete TCP IPC server for Signal that listens for JSON-line envelopes from Pulse, deserialises them, routes them to domain handlers via the existing `IpcRouter`, and sends replies back to clients.

## Files Added

### Core IPC Infrastructure

- `src/ipc/IpcEnvelope.hpp` / `src/ipc/IpcEnvelope.cpp`
  - New typed IPC envelope structure matching Chorus spec
  - Enums for `IpcOrigin`, `IpcTarget`, `IpcKind`, `IpcPriority`
  - Conversion helpers to/from strings
  - ISO 8601 timestamp generation utility

- `src/ipc/IpcEnvelopeCodec.hpp` / `src/ipc/IpcEnvelopeCodec.cpp`
  - JSON serialisation and deserialisation of `IpcEnvelope`
  - Line-delimited JSON format (one envelope per line)
  - Validation of required fields and enum values
  - Graceful error handling for malformed input

### TCP Server Components

- `src/ipc/TcpClientSession.hpp` / `src/ipc/TcpClientSession.cpp`
  - Per-client TCP session handler
  - Asynchronous line-by-line reading using Asio
  - Thread-safe envelope sending
  - Automatic cleanup on disconnect

- `src/ipc/TcpServer.hpp` / `src/ipc/TcpServer.cpp`
  - TCP server accepting multiple clients
  - Listens on configurable host/port (default: `127.0.0.1:8787`)
  - Tracks active client sessions via weak pointers
  - Graceful shutdown support

- `src/ipc/DomainDispatcher.hpp` / `src/ipc/DomainDispatcher.cpp`
  - Central dispatcher bridging new typed envelopes with existing `IpcRouter`
  - Routes envelopes to appropriate domain handlers
  - Sends acknowledgement events for commands
  - Sends error responses for unknown domains

### Build and Dependencies

- Updated `CMakeLists.txt` to fetch:
  - `nlohmann/json` v3.11.3 for JSON parsing
  - Standalone Asio (header-only) for TCP networking

- Updated `src/CMakeLists.txt` to:
  - Include new source files
  - Link with nlohmann/json
  - Configure standalone Asio with `ASIO_STANDALONE` define
  - Link system libraries (CoreFoundation on macOS, pthread on Linux)

### Tests

- `tests/test_ipc_envelope_codec.cpp`
  - Unit tests for envelope serialisation/deserialisation
  - Tests for correlation IDs, error handling, invalid input

## Files Modified

- `src/core/SignalApp.cpp`
  - Replaced stub `run()` implementation with full TCP server integration
  - Reads host/port from environment variables (`SIGNAL_HOST`, `SIGNAL_PORT`) or defaults
  - Sets up signal handling (SIGINT/SIGTERM) for graceful shutdown
  - Starts IO context and TCP server, runs event loop

- `src/CMakeLists.txt`
  - Added new IPC source files to `signal-core` library
  - Configured Asio and nlohmann/json dependencies

- `tests/CMakeLists.txt`
  - Added `test_ipc_envelope_codec.cpp` to test executable

- `CHANGELOG.md`
  - Added entry for IPC server implementation

## Architecture

### Envelope Flow

```
TCP Client (Pulse)
    ↓ (sends JSON line)
TcpClientSession::doRead()
    ↓ (parses line)
deserialiseEnvelope()
    ↓ (validates, creates IpcEnvelope)
DomainDispatcher::handleEnvelope()
    ↓ (routes by domain)
IpcRouter::dispatch()
    ↓ (calls domain handler)
EngineDomain / TransportDomain
    ↓ (for commands, sends reply)
TcpClientSession::send()
    ↓ (serialises, sends JSON line)
TCP Client (Pulse)
```

### Server Startup

1. `SignalApp::run()` creates `asio::io_context`
2. Constructs `DomainDispatcher` with existing `IpcRouter`
3. Creates `TcpServer` on host/port (from env or defaults)
4. Registers signal handler for SIGINT/SIGTERM
5. Starts server and runs IO loop
6. Server logs: `[Signal] IPC server listening on 127.0.0.1:8787`

### Configuration

- Default host: `127.0.0.1`
- Default port: `8787`
- Environment variables: `SIGNAL_HOST`, `SIGNAL_PORT`

## Namespace

All new IPC code is in the `loophole::signal::ipc` namespace to avoid conflicts with POSIX `signal()` function.

## Integration with Existing Code

The implementation maintains backward compatibility with the existing `IpcRouter` and domain handlers by:
- Converting new typed `IpcEnvelope` to legacy `Envelope` struct when dispatching
- Preserving all existing domain handler interfaces
- No changes required to existing domain handlers

## Testing

All tests pass:
- Envelope serialisation/deserialisation
- Correlation ID handling
- Error envelope handling
- Invalid input rejection

## Future TODOs

1. **Security and Hardening**
   - Authentication/authorisation for clients
   - Rate limiting per client
   - Connection timeouts
   - Maximum message size limits

2. **Domain Implementations**
   - Complete engine domain command/event handling
   - Complete transport domain command/event handling
   - Add more domain handlers as needed

3. **Performance**
   - Connection pooling if needed
   - Message batching for high-rate streams
   - Binary fast-path support (as per Chorus spec)

4. **Configuration**
   - CLI argument parsing for host/port
   - Config file support
   - TLS/SSL support for production

5. **Monitoring**
   - Connection count metrics
   - Message rate statistics
   - Error rate tracking

6. **Error Handling**
   - More detailed error codes
   - Error context in replies
   - Client-specific error recovery

