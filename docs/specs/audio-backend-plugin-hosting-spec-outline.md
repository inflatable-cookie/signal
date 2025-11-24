# Audio Backend & Plugin Hosting – Spec Outline

> **Status**: Draft outline (to be expanded into full spec)  
> **Related ADR**: ADR-000X – Audio Backend & Plugin Hosting Strategy (No JUCE, Backend-Agnostic Engine)  
> **Scope**: Signal (audio engine), with implications for Pulse (model) and Aura (UI)

---

## 1. Purpose & Scope

This document outlines the design for:

1. The **AudioBackend** abstraction in **Signal**, responsible for:
   - Talking to OS audio/MIDI devices via small, focused libraries (e.g. `miniaudio`).
   - Providing a single audio callback entry point for the engine (`renderBlock`).

2. The **PluginHost** and **PluginInstance** abstractions in **Signal**, responsible for:
   - Discovering, loading, and managing third-party plugins.
   - Presenting a **format-agnostic** interface to the rest of the engine.
   - Supporting multiple plugin formats over time (CLAP, VST3, AU, LV2, etc).

This outline is intentionally high-level. A follow-up spec will fill in detailed API shapes, threading diagrams, and examples.

---

## 2. Goals & Non-Goals

### 2.1 Goals

- Provide a **backend-agnostic engine core**:
  - Engine logic (scheduling, mixing, automation, looping, metering, plugin graph) is independent of any particular audio device API or framework.
- Define a small, explicit **AudioBackend interface**:
  - Allows swapping implementations (e.g. `MiniaudioBackend`, future `JackBackend`, or even `JuceBackend`) without impacting the engine core.
- Define a **PluginHost abstraction**:
  - Format-neutral representation of plugins, parameters, and state.
  - Separate adapter modules for CLAP, VST3, AU, LV2, and optionally VST2.
- Enable **incremental adoption of plugin formats**:
  - CLAP and VST3 first, AU/LV2 later.
- Keep architecture amenable to **process isolation / bridging** in future:
  - e.g. Intel plugins under Rosetta, per-plugin host processes.

### 2.2 Non-Goals (for this spec outline)

- Full API reference of each class/function.
- Detailed plugin format-specific behaviour (that will be in per-format specs).
- Detailed GUI embedding strategy for plugin editors.
- Detailed host-side management of sandboxed processes.

---

## 3. Architectural Overview

### 3.1 High-level data flow

At a high level, the run-time audio path is:

```text
OS Audio Device(s)
   ↑↓   (via AudioBackend, e.g. MiniaudioBackend)
Signal Engine (EngineHost)
   - Transport / timebase
   - Session schedule (from Pulse)
   - Mixer (gain / mute / solo / pan)
   - Automation curves
   - Plugin graph (hosted by PluginHost)
   - Metering & diagnostics
```

- **Pulse**:
  - Owns the session model (tracks, clips, automation, mixer).
  - Builds schedules and sends them to Signal over IPC.
- **Signal**:
  - Owns the real-time engine, including plugin hosting and audio device interaction.
- **Aura**:
  - UI only; does not talk to audio devices or plugins directly.

### 3.2 Module boundaries (Signal)

Proposed Signal modules:

- `signal/core` – engine core and `EngineHost`.
- `signal/backend` – `AudioBackend` interface and implementations (`MiniaudioBackend`, etc.).
- `signal/plugins` – `PluginHost`, `PluginInstance`, plugin graph.
- `signal/plugins/*` – per-format adapters (`clap`, `vst3`, `au`, `lv2`, `vst2-legacy`).
- `signal/tests` – engine, backend, and plugin host tests.

---

## 4. AudioBackend Module

### 4.1 Responsibilities

- Manage OS-level audio (and possibly MIDI) devices:
  - Enumeration, selection, opening, closing.
  - Buffer sizes, sample rate negotiation.
- Provide a single **audio callback** that:
  - Receives input buffers (where applicable).
  - Calls the engine’s **renderBlock** function with:
    - A context (sampleRate, blockSize, host time).
    - In/out audio buffers.
- Handle recoverable device errors:
  - Lost device, buffer underrun notifications, etc.
- Do **no engine logic**:
  - No mixing, no scheduling, no plugin calls.
  - Those belong in `EngineHost`.

### 4.2 Interface sketch

(Exact types to be finalised in the full spec.)

```cpp
struct EngineRenderContext {
    double hostTimeSeconds;
    double sampleRate;
    int blockSize;
    // transport snapshot, loop info, etc. may be included or fetched by EngineHost
};

class AudioBackend {
public:
    virtual ~AudioBackend() = default;

    virtual bool initialise(const AudioBackendConfig& config) = 0;
    virtual void shutdown() = 0;

    virtual bool start() = 0;
    virtual void stop() = 0;

    // Engine provides this callback; backend calls it from the audio thread:
    virtual void setRenderCallback(
        std::function<void(EngineRenderContext&, AudioBus&, AudioBus&)> cb
    ) = 0;
};
```

### 4.3 MiniaudioBackend

Initial implementation:

- Wraps **miniaudio** for:
  - Device selection (input/output).
  - Audio callback registration.
- Responsibilities:
  - Translate miniaudio’s callback signature into `EngineRenderContext` + `AudioBus`.
  - Manage sample rate & buffer size negotiation.
- Must obey real-time constraints:
  - No allocations, no locks, no logging inside callback.

### 4.4 Future backends

- Additional `AudioBackend` implementations can be added later:
  - `JackBackend` (Linux JACK).
  - `AsioBackend` (explicit ASIO integration).
  - `JuceBackend` if ever required, behind the same interface.
- The engine core must not know which backend is in use.

---

## 5. EngineHost & renderBlock

### 5.1 Role of EngineHost

`EngineHost` is the engine’s main façade from the audio-thread perspective:

- Owns or has access to:
  - Transport state (position, loop).
  - Current schedule (tracks & clips).
  - Mixer state.
  - Automation curves.
  - Plugin graph.
- Provides:
  - `renderBlock` – invoked from the audio backend each audio callback.

### 5.2 Responsibilities in renderBlock

Per audio block (render call):

1. Read current **transport and loop** state.
2. Determine **time range** for this block (start/end in samples/beats).
3. Determine **active clips** in the block from schedule.
4. For each track:
   - Sum relevant clips into a track bus.
   - Run plugins in the track’s plugin chain.
   - Apply mixer gain/mute/solo + automation.
   - Apply pan (with automation) to route to output channels.
5. Sum tracks to master bus(es).
6. Update **metering**.
7. Write final audio into output buffer.

Implementation details (schedule, mixer, automation, plugin graph) are delegated to other modules; `EngineHost` orchestrates.

---

## 6. PluginHost & PluginInstance

### 6.1 Responsibilities

The **PluginHost** layer is responsible for:

- Discovering plugins on disk (per format).
- Exposing a **format-agnostic view** of plugins to the rest of Signal.
- Creating, initialising, and destroying plugin instances.
- Providing a unified interface for:
  - Audio/MIDI processing.
  - Parameters (static and automatable).
  - State save/restore.

The core engine must not access CLAP/VST3/AU/LV2 APIs directly; it uses `PluginHost` and `PluginInstance`.

### 6.2 Format-agnostic data model

Key concepts:

- `PluginDescriptor`:
  - Identifies a plugin addressably and persistently (for sessions).
- `PluginInstance`:
  - A single loaded instance of a plugin.
- `ParamId`:
  - Stable, format-neutral parameter identifier.

### 6.3 Interface sketch – PluginDescriptor

```cpp
struct PluginDescriptor {
    std::string id;       // stable ID for session serialization
    std::string name;
    std::string vendor;
    std::string format;   // "clap", "vst3", "au", "lv2", "vst2-legacy"

    // capabilities, bus configuration, categories, etc.
    // will be expanded in the full spec.
};
```

### 6.4 Interface sketch – PluginInstance

```cpp
class PluginInstance {
public:
    virtual ~PluginInstance() = default;

    virtual void prepare(double sampleRate, int blockSize) = 0;
    virtual void reset() = 0;

    virtual void process(const AudioBus& in,
                         AudioBus& out,
                         const EventQueue& events) = 0;

    virtual void setParameter(ParamId id, float normalisedValue) = 0;
    virtual float getParameter(ParamId id) const = 0;

    virtual void getState(Blob& out) = 0;
    virtual void setState(const Blob& in) = 0;

    // GUI/editor integration will be specified later
};
```

### 6.5 Interface sketch – PluginHost

```cpp
class PluginHost {
public:
    virtual ~PluginHost() = default;

    virtual std::vector<PluginDescriptor> scan() = 0;

    virtual std::unique_ptr<PluginInstance>
    createInstance(const PluginDescriptor& desc) = 0;
};
```

The engine will maintain a **plugin graph** (per track, per bus) using `PluginInstance` objects.

---

## 7. Format Adapters (CLAP, VST3, AU, LV2, VST2)

Each plugin format gets its own adapter module implementing `PluginHost` and `PluginInstance`.

### 7.1 CLAP

- Module: `signal/plugins/clap/`
- Uses CLAP SDK.
- Responsibilities:
  - Discover CLAP plugins.
  - Wrap CLAP descriptors into `PluginDescriptor`.
  - Implement `ClapPluginInstance` with:
    - Audio/MIDI processing.
    - Parameter mapping.
    - State save/restore.

### 7.2 VST3

- Module: `signal/plugins/vst3/`
- Uses VST3 SDK from Steinberg.
- Similar responsibilities as CLAP adapter.

### 7.3 AU (macOS)

- Module: `signal/plugins/au/`
- Uses Apple Audio Unit APIs.
- Focused initially on instrument/effect units only.

### 7.4 LV2 (Linux)

- Module: `signal/plugins/lv2/`
- Uses LV2 hosting libraries.

### 7.5 VST2 (legacy)

- Module: `signal/plugins/vst2-legacy/`
- Only if licensing allows and it is worth the effort.
- Potentially lower priority or separate build option.

---

## 8. Pulse & Aura Interactions

### 8.1 Pulse ↔ Signal (plugins)

Pulse must:

- Store plugin instances as **format-neutral descriptors + state** in the session model:
  - `pluginFormat`, `pluginId`, `instanceId`, parameter automation, state blobs.
- Provide schedules and parameter automation for:
  - Track inserts, sends, and bus plugins.
- Send plugin graph and state changes to Signal over IPC:
  - Create/destroy instance.
  - Change parameter.
  - Set automation curves.
  - Request state snapshot.

Signal must:

- Expose plugin graph state to Pulse in an agreed snapshot format.
- Apply plugin graph changes from Pulse safely on control threads.

### 8.2 Aura ↔ Pulse (plugins)

Aura (UI) will:

- Show plugin slots, param names, and automation curves using Pulse data.
- Send user actions (add plugin, remove plugin, change parameter, open editor) to Pulse.
- Never talk directly to plugin formats or engine; it always goes via Pulse.

---

## 9. Testing & Diagnostics

### 9.1 Engine core + AudioBackend

- Unit tests:
  - Offline render tests for EngineHost with synthetic schedules.
  - Timebase, loop, mixer, automation interactions.
- Integration tests:
  - Engine & MiniaudioBackend working together in a test harness (where feasible).

### 9.2 PluginHost

- Unit tests per adapter:
  - Loading a known set of test plugins.
  - Parameter get/set round-trips.
  - State save/restore.
  - Basic process() calls on test audio.

- Cross-format tests:
  - Ensuring PluginInstance behaviour is consistent across CLAP/VST3/AU/LV2 where semantics overlap.

### 9.3 Diagnostics

- Debug logging (control thread only) for:
  - Plugin discovery and load/unload events.
  - Backend device changes.
- Engine-level diagnostics:
  - CPU usage, block utilisation.
  - Plugin invocation timing.
- Future:
  - Integration with Aura’s diagnostics panel via Pulse.

---

## 10. Phased Implementation Plan (Outline)

This spec outlines **phases**, not exact tickets:

1. **Phase A – EngineCore + AudioBackend**
   - Finalise `EngineHost` and `AudioBackend` interfaces.
   - Implement `MiniaudioBackend`.
   - Wire basic renderBlock with internal generator to prove audio output.

2. **Phase B – PluginHost skeleton**
   - Implement `PluginHost` / `PluginInstance` interfaces.
   - Add scaffolding for plugin graph (without formats yet).

3. **Phase C – CLAP adapter**
   - Implement `ClapPluginHost` and `ClapPluginInstance`.
   - Integrate with EngineHost plugin graph for inserts.

4. **Phase D – VST3 adapter**
   - Implement VST3 hosting behind the same abstractions.
   - Ensure CLAP and VST3 can co-exist in one project.

5. **Phase E – AU / LV2**
   - Implement AU (macOS) and LV2 (Linux) adapters.
   - Extend test coverage.

6. **Phase F – Isolation / bridging (later ADRs)**
   - Define process boundaries and IPC for plugin isolation.
   - Add optional per-plugin host processes.

Each phase will produce its own detailed spec (or extended sections in this one) and its own report in Chorus.

---

## 11. Open Questions

- How do we want to model complex plugin I/O topologies (multi-out, sidechain) in a format-agnostic way?
- What is our minimum bar for host–plugin isolation in v1 (same process vs separate process)?
- How will plugin editor GUIs be integrated with Aura (likely via a small native window bridge and IPC)?
- How do we want to handle plugin scanning:
  - On startup vs on-demand?
  - Per-format or unified?

These will be addressed in follow-up specs and ADRs once this outline is accepted.
