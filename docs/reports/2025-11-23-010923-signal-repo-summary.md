# Repository Summary: Signal

**Generated:** 2025-01-27  
**Purpose:** Comprehensive summary of Signal repository state, architecture alignment, IPC consistency, and code quality

---

## 1. Directory Structure

```
signal/
├── AGENTS.md
├── CHANGELOG.md
├── CMakeLists.txt
├── LICENSE
├── README.md
├── docs/
│   ├── plans/
│   │   └── implementation.md
│   └── reports/
│       └── 2025-11-22-215413-signal-ipc-server.md
├── src/
│   ├── CMakeLists.txt
│   ├── main.cpp
│   ├── core/
│   │   ├── EngineHost.cpp
│   │   ├── EngineHost.hpp
│   │   ├── SignalApp.cpp
│   │   └── SignalApp.hpp
│   ├── domains/
│   │   ├── EngineDomain.cpp
│   │   ├── EngineDomain.hpp
│   │   ├── TransportDomain.cpp
│   │   └── TransportDomain.hpp
│   └── ipc/
│       ├── DomainDispatcher.cpp
│       ├── DomainDispatcher.hpp
│       ├── Envelope.cpp
│       ├── Envelope.hpp
│       ├── IpcEnvelope.cpp
│       ├── IpcEnvelope.hpp
│       ├── IpcEnvelopeCodec.cpp
│       ├── IpcEnvelopeCodec.hpp
│       ├── Router.cpp
│       ├── Router.hpp
│       ├── TcpClientSession.cpp
│       ├── TcpClientSession.hpp
│       ├── TcpServer.cpp
│       └── TcpServer.hpp
└── tests/
    ├── CMakeLists.txt
    ├── test_envelope.cpp
    ├── test_ipc_envelope_codec.cpp
    └── test_router.cpp
```

---

## 2. Folder Purposes

### Root Level

- **`AGENTS.md`** — AI agent guidance for Signal development (formatting rules, IPC consistency, real-time safety, changelog discipline)
- **`CHANGELOG.md`** — Unreleased entries for significant changes
- **`CMakeLists.txt`** — Main CMake configuration, dependency fetching (nlohmann/json, Asio), test setup
- **`README.md`** — Overview of Signal’s role as real-time audio engine, responsibilities, architecture integration
- **`LICENSE`** — MIT licence with Loophole name protection clause

### `src/`

- **`core/`** — Core application types (`SignalApp`, `EngineHost`)
- **`domains/`** — Per-domain IPC handlers (`EngineDomain`, `TransportDomain`)
- **`ipc/`** — IPC transport, envelope parsing, codec, router, TCP server/client session management
- **`main.cpp`** — Entry point, creates and runs `SignalApp`

### `tests/`

- Unit tests using Catch2 framework
- Tests for envelope codec, router, basic envelope structure

### `docs/`

- **`plans/`** — Implementation plan (milestones, architecture notes)
- **`reports/`** — Historical analysis reports (one report: IPC server implementation)

**Structure Assessment:** Clean and matches expected Signal layout. Architecture boundaries are respected: core owns app lifecycle, IPC owns transport, domains own handler logic.

---

## 3. IPC & Spec Alignment

### 3.1 Envelope Structure

**Status:** ✅ **Mostly compliant, but migration in progress**

Signal has **two envelope structures**:

1. **`IpcEnvelope`** (new, typed) — Located in `src/ipc/IpcEnvelope.hpp`
   - Uses full field names internally: `version`, `timestamp`, `correlationId`
   - Properly typed enums: `IpcOrigin`, `IpcTarget`, `IpcKind`, `IpcPriority`
   - Correctly serialises/deserialises to/from abbreviated JSON fields (`v`, `id`, `cid`, `ts`) per Chorus spec

2. **`Envelope`** (legacy, string-based) — Located in `src/ipc/Envelope.hpp`
   - Uses abbreviated field names: `v`, `id`, `cid`, `ts`
   - String-based fields for `kind`, `priority`, `origin`, `target`
   - Still used by `Router` and domain handlers

**Issue:** Migration is incomplete. `DomainDispatcher` converts `IpcEnvelope` → `Envelope` to bridge to existing router. Domain handlers (`EngineDomain`, `TransportDomain`) still use the legacy `Envelope` struct.

### 3.2 Envelope Fields

**JSON serialisation:** ✅ **Correct**
- Uses abbreviated field names in JSON (`v`, `id`, `cid`, `ts`) matching Chorus spec
- Codec correctly maps between JSON (abbreviated) and internal `IpcEnvelope` (full names)

**Required fields:** ✅ **Validated**
- Codec validates all required fields (`v`, `id`, `ts`, `origin`, `target`, `domain`, `kind`, `name`, `priority`, `payload`)
- Handles optional `cid` and `error` fields correctly

**Kind values:** ✅ **Correct**
- Supports: `command`, `event`, `snapshot`, `error` (matches Chorus spec)

**Priority values:** ✅ **Correct**
- Supports: `realtime`, `high`, `normal`, `low` (matches Chorus spec)

**Origin/Target values:** ✅ **Correct**
- Supports: `aura`, `pulse`, `signal`, `composer` (matches Chorus spec)

### 3.3 Domain Names

**Current domains:** `engine`, `transport`

**Chorus spec domains for Signal:**
- `engine` (Signal Engine domain)
- `transport` (Signal Transport domain)
- `graph` (Signal Graph domain — not yet implemented)
- `hardware` (Signal Hardware domain — not yet implemented)
- `plugin` (Signal Plugin domain — not yet implemented)
- `media` (Signal Media domain — not yet implemented)
- `diagnostics` (Signal Diagnostics domain — not yet implemented)

**Status:** ✅ **Domain names match Chorus spec** — Signal uses unqualified domain names (`engine`, `transport`) which is correct; the `signal.` prefix appears in spec doc filenames/context but not in the envelope `domain` field.

**Missing domains:** Several domains defined in Chorus are not yet implemented (graph, hardware, plugin, media, diagnostics).

### 3.4 Message Naming

**Current implementation:** Domain handlers check `env.name` strings directly (e.g., `"start"`, `"stop"` in `EngineDomain`).

**Chorus spec:** Defines specific command/event names per domain (e.g., `engine.start`, `engine.stop`, `engine.handshake`, `transport.play`, `transport.seek`).

**Status:** ⚠️ **Partial compliance** — Basic commands exist (`start`, `stop`, `reset`, `shutdown` in Engine; likely similar in Transport), but many Chorus-specified commands/events are not yet implemented (e.g., `engine.handshake`, `engine.configure`, `transport.setLoopRegion`).

### 3.5 Correlation IDs

**Implementation:** ✅ **Correct**
- `IpcEnvelope` has `std::optional<std::string> correlationId`
- Codec correctly handles `cid` field (null or string)
- `DomainDispatcher` sets correlation ID in reply events

### 3.6 Timestamps

**Implementation:** ✅ **Correct**
- `currentTimestamp()` generates ISO 8601 format (`YYYY-MM-DDTHH:MM:SS.sssZ`)
- Timestamp is required and validated in codec

---

## 4. Code Quality & Consistency

### 4.1 Formatting Rules (AGENTS.md)

**Function parameter formatting:** ⚠️ **Partial compliance**

Some functions follow the rule (one param per line when multi-line), but several places need checking:
- `TcpServer::TcpServer()` — parameters could be better formatted
- `DomainDispatcher::handleEnvelope()` — acceptable
- `IpcEnvelopeCodec` functions — acceptable

**Flow-control spacing:** ⚠️ **Needs review**

AGENTS.md requires blank lines before/after flow statements inside blocks (unless first/last). Many places likely comply, but needs systematic check.

**Multi-line boolean conditions:** ⚠️ **Needs review**

AGENTS.md requires logical operators at end of line for multi-clause conditions. Needs verification.

### 4.2 Naming Conventions

**Status:** ✅ **Consistent**
- C++ naming follows conventions (PascalCase for classes, camelCase for members with `_` prefix, snake_case for locals)
- IPC field names match Chorus spec (abbreviated in JSON, full names internally)

### 4.3 Architecture Boundaries

**Status:** ✅ **Respected**
- `core/` — App lifecycle only (no IPC logic)
- `ipc/` — Transport and envelope parsing only (no domain business logic)
- `domains/` — Domain handler logic only (no transport details)

### 4.4 Dead Code

**Potential issues:**
- Legacy `Envelope` struct still exists and is actively used by router and domain handlers
- This is intentional (migration bridge), but should be removed once migration completes

### 4.5 Error Handling

**Status:** ✅ **Good**
- Uses structured error types (`IpcErrorInfo`)
- Codec validates and returns `std::optional` for invalid input
- Domain dispatcher sends error responses for unknown domains

### 4.6 Real-Time Safety

**Status:** ✅ **Architecturally sound** (not yet implemented)

- All IPC and JSON parsing happens on non-real-time threads (TCP server, Asio IO context)
- Real-time audio code would be in `EngineHost` (currently stub)
- Separation is correct: IPC layer is clearly separated from audio processing

---

## 5. Cross-Repo Consistency

### 5.1 IPC Envelope Fields

**Signal vs Pulse vs Aura:**

| Field | Signal | Pulse | Aura | Match |
|-------|--------|-------|------|-------|
| `v` | ✅ (JSON) | ✅ | ✅ | ✅ |
| `id` | ✅ (JSON) | ✅ | ✅ | ✅ |
| `cid` | ✅ (JSON) | ✅ | ✅ | ✅ |
| `ts` | ✅ (JSON) | ✅ | ✅ | ✅ |
| `origin` | ✅ | ✅ | ✅ | ✅ |
| `target` | ✅ | ✅ | ✅ | ✅ |
| `domain` | ✅ | ✅ | ✅ | ✅ |
| `kind` | ✅ | ✅ | ✅ | ✅ |
| `name` | ✅ | ✅ | ✅ | ✅ |
| `priority` | ✅ | ✅ | ✅ | ✅ |
| `payload` | ✅ | ✅ | ✅ | ✅ |
| `error` | ✅ | ✅ | ✅ | ✅ |

**Status:** ✅ **All repos match Chorus spec for envelope fields**

### 5.2 Domain Names

**Pulse domains:** `client`, `debug`, `engine`, `project`, `track`, `transport` (Pulse-owned domains)

**Signal domains:** `engine`, `transport` (Signal-owned domains, unqualified as per spec)

**Aura domains:** Uses same domain names as Pulse (client-side projection)

**Status:** ✅ **Consistent** — Signal uses unqualified domain names which is correct (Pulse routes based on target).

### 5.3 Implementation Status

**Pulse:** Has full IPC server, envelope parsing, domain dispatcher, multiple domain handlers

**Signal:** Has TCP IPC server, envelope parsing, domain dispatcher, stub domain handlers (`engine`, `transport`)

**Aura:** Has IPC client (Electron preload/renderer), envelope creation, domain stores

**Status:** ⚠️ **Signal is behind Pulse/Aura** — Infrastructure is in place, but domain implementations are stubs.

---

## 6. Test Coverage

**Current tests:**
- `test_envelope.cpp` — Basic envelope structure
- `test_ipc_envelope_codec.cpp` — Envelope serialisation/deserialisation, correlation IDs, error handling
- `test_router.cpp` — Router dispatch logic

**Status:** ✅ **Good coverage for IPC layer**

**Missing tests:**
- Domain handler logic (EngineDomain, TransportDomain)
- TCP server/client session handling (integration tests)
- Error scenarios (malformed envelopes, connection failures)
- Real-time safety boundaries (ensuring IPC never blocks RT threads)

---

## 7. Suggested Improvements / Next Steps

### Priority 1: Complete Envelope Migration

1. **Remove legacy `Envelope` struct** — Migrate `Router` and all domain handlers to use `IpcEnvelope` directly, then remove `Envelope.hpp/cpp`.
   - Update `Router` to accept `IpcEnvelope` instead of `Envelope`
   - Update `EngineDomain` and `TransportDomain` to use `IpcEnvelope`
   - Remove `DomainDispatcher::handleEnvelope()` conversion logic
   - Delete `Envelope.hpp` and `Envelope.cpp`

### Priority 2: Implement Chorus-Specified Commands/Events

2. **Engine domain** — Implement full Chorus spec commands/events:
   - `engine.handshake` (protocol version negotiation)
   - `engine.configure` (sample rate, block size, processing mode)
   - `engine.start` / `engine.stop` / `engine.restart`
   - `engine.stateChanged` events
   - `engine.configResolved` events
   - Error handling per spec

3. **Transport domain** — Implement full Chorus spec commands/events:
   - `transport.play` / `transport.pause` / `transport.stop`
   - `transport.seek` (sample-accurate positioning)
   - `transport.setLoopRegion` / `transport.setPreRoll`
   - `transport.stateChanged` / `transport.positionUpdate` events
   - Loop/record state handling

### Priority 3: Add Missing Domains

4. **Graph domain** — Implement `signal.graph` domain (graph snapshots, deltas, channel/node lifecycle)

5. **Hardware domain** — Implement `signal.hardware` domain (device enumeration, selection, configuration)

### Priority 4: Code Quality & Testing

6. **Formatting compliance** — Audit and fix all function parameter formatting, flow-control spacing, multi-line boolean conditions per AGENTS.md

7. **Test coverage** — Add integration tests for TCP server, domain handler logic, error scenarios

8. **Real-time safety verification** — Add tests/checks to ensure IPC layer never blocks real-time audio threads

### Priority 5: Configuration & Hardening

9. **CLI arguments** — Add argument parsing for host/port (currently environment variables only)

10. **Connection security** — Add authentication/authorisation, rate limiting, connection timeouts

11. **Error recovery** — Improve error handling and client-specific error recovery

---

## 8. Summary

Signal has a **solid foundation** with:
- ✅ Correct IPC envelope structure matching Chorus spec
- ✅ Clean architecture separation (core/IPC/domains)
- ✅ TCP server infrastructure in place
- ✅ Good test coverage for IPC codec

**Main gaps:**
- ⚠️ Incomplete envelope migration (legacy `Envelope` still in use)
- ⚠️ Stub domain implementations (need full Chorus-specified commands/events)
- ⚠️ Missing domains (graph, hardware, plugin, media, diagnostics)
- ⚠️ Formatting compliance needs verification

**Overall assessment:** Signal is in early but well-structured state. Infrastructure aligns with Chorus specs, but domain logic needs significant implementation work to match specification completeness of Pulse and Aura.
