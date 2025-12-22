# Signal IPC Surface

This document describes the IPC (Inter-Process Communication) interface exposed by Signal, the Loophole DAW audio engine.

## Overview

Signal communicates with Pulse (the model/controller) via TCP using `IpcEnvelope` messages. All IPC is handled through:

- `IpcEnvelope` - Canonical message format
- `DomainDispatcher` - Routes envelopes to domain handlers
- `IpcDomainHandler` - Interface implemented by each domain

The legacy `Router` and `Envelope` types have been completely removed. All IPC now follows the single path:

> TCP → `IpcEnvelope` (decode) → `DomainDispatcher` → `IpcDomainHandler` → domain logic → `IpcEnvelope` (encode) → TCP

## Domains

Signal exposes the following IPC domains:

### engine

Commands (client → Signal):
- `start` - Start the audio engine
- `stop` - Stop the audio engine
- `reset` - Reset engine state
- `shutdown` - Shutdown the engine
- `heartbeat` - Request heartbeat response
- `scheduleSession` / `playbackScheduleSnapshot` - Set playback schedule
- `graphSnapshot` / `applyGraphSnapshot` - Apply audio graph snapshot

Events (Signal → client):
- `state` - Engine state change (lifecycle: "stopped" | "starting" | "running" | "error")
- `heartbeat` - Heartbeat response

### transport

Commands (client → Signal):
- `play` - Start playback
- `stop` - Stop playback (optionally with position)
- `seek` - Seek to position (samples, seconds, or beats)
- `setLoopEnabled` - Enable/disable loop
- `setLoopRegion` - Set loop region (samples, seconds, or beats)
- `setTempo` - Set tempo in BPM

Events (Signal → client):
- `state` - Transport state (isPlaying, positionBeats, loopEnabled, loopRegion)

### hardware

Commands (client → Signal):
- `refreshOutputDevices` - List available output devices
- `selectOutputDevice` - Select output device

Events (Signal → client):
- `state` - Hardware state (outputDevices, activeDeviceId, optional lastError)

### mixer

Commands (client → Signal):
- `updateChannel` - Update channel parameters (gain, pan, mute, solo)

### automation

Commands (client → Signal):
- `setCurvesForSession` - Set automation curves
- `automationSnapshot` - Load automation snapshot
- `updateCurve` - Update single automation curve

### assets

Commands (client → Signal):
- `registerAudioAsset` - Register audio asset for playback

### metering

Events (Signal → client):
- Metering events are published automatically (no commands currently)

## Message Format

All messages use the `IpcEnvelope` format:

```json
{
  "v": 1,
  "id": "unique-message-id",
  "cid": "correlation-id-optional",
  "ts": "2025-01-01T00:00:00Z",
  "origin": "pulse" | "aura" | "signal" | "composer",
  "target": "pulse" | "aura" | "signal" | "composer",
  "domain": "engine" | "transport" | "hardware" | "mixer" | "automation" | "assets" | "metering",
  "kind": "command" | "event" | "snapshot" | "error",
  "name": "command-or-event-name",
  "priority": "low" | "normal" | "high",
  "payload": {}
}
```

## Implementation Notes

- All domains implement `IpcDomainHandler` interface
- `DomainDispatcher` maintains a registry of domain handlers
- No legacy `Router` or `Envelope` types are used
- All IPC is synchronous within the IPC thread context
- Domain handlers update `EngineHost` state directly
- Events are sent back via `TcpClientSession`
