# Signal Skeleton Implementation Status

**Date:** 2025-11-23 03:00:00 UTC  
**Status:** Foundation Complete, Event Processing Remaining

## Overview

This document summarises the current state of the Signal skeleton implementation and Pulse ↔ Signal integration. The foundation is in place, but Signal event processing in Pulse needs to be wired up.

## Completed Components

### Signal Repository

1. **Port Configuration**
   - Updated default port from 8787 to 7888
   - Signal listens on TCP port 7888

2. **Audio Thread Skeleton**
   - Created `AudioThread` class with minimal implementation
   - Integrated into `EngineHost`
   - Audio thread runs independently, generates silence/test tone
   - Thread-safe state communication ready

3. **Event Sending**
   - Updated `DomainDispatcher` to send `engine.state` events after processing engine commands
   - Events include `lifecycle` field matching Pulse's format
   - Events are correlated with command IDs

4. **Documentation**
   - Updated `AGENTS.md` with audio thread rules and real-time safety guidelines
   - Updated `implementation.md` with audio thread integration details

### Pulse Repository

1. **Signal Coordinator Integration**
   - SignalCoordinator stored in RuntimeConfig
   - EngineDomain forwards `engine.start` and `engine.stop` commands to Signal
   - Port updated to 7888 to match Signal

2. **Command Forwarding**
   - Engine commands forwarded from Pulse → Signal
   - Infrastructure ready for transport command forwarding

3. **Process Management**
   - Signal supervisor spawns and manages Signal process
   - Graceful shutdown handling in place

### Aura Repository

1. **Debug Panel Updates**
   - Engine panel shows lifecycle state
   - Diagnostics display ready (CPU load, xruns)
   - Engine controls (Start/Stop) functional

2. **State Tracking**
   - Engine state store includes diagnostics support
   - Handlers ready for `engine.state` and `engine.diagnostics` events

## Remaining Work

### Critical: Signal Event Processing in Pulse

The Signal coordinator receives events from Signal on a separate thread, but these events need to:

1. Update Pulse's `SessionState.engine` state
2. Forward `engine.state` events to Aura via Pulse's normal IPC flow

**Current Challenge:**
- Signal events come through SignalConnection (separate thread)
- SessionState is managed per-client in Pulse's server loop
- Need mechanism to inject Signal events into Pulse's event processing

**Suggested Approach:**
- Store Signal events in a shared queue/channel
- Pulse server loop periodically processes Signal events
- Or: Make engine state shared and have Signal coordinator update it directly

### Transport Command Forwarding

- Add transport command forwarding to Signal coordinator (similar to engine commands)
- Forward `transport.play`, `transport.stop` from Pulse → Signal

### Signal Diagnostics Events

- Signal should send periodic `engine.diagnostics` events
- Include CPU load and xrun count
- Pulse should forward these to Aura

## Architecture Notes

### Current Flow (Working)
1. Aura → Pulse: `engine.start` command
2. Pulse → Signal: Forward `engine.start` command
3. Signal: Process command, update EngineHost state
4. Signal → Pulse: Send `engine.state` event (CORRELATED, via cid)

### Current Flow (Needs Implementation)
4. Pulse: Receive Signal event, update SessionState
5. Pulse → Aura: Forward `engine.state` event

The missing piece is step 4-5: processing Signal events and updating/forwarding.

## Next Steps

1. Implement Signal event processing in Pulse
2. Wire Signal coordinator callback to update session state
3. Forward transport commands to Signal
4. Add periodic diagnostics events from Signal
5. Test end-to-end flow

The foundation is solid; the remaining work is wiring up event processing.

