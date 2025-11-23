# Decision: Signal Threading and Concurrency Model

- **ID:** 2025-11-23-signal-threading-and-concurrency  
- **Date:** 2025-11-23  
- **Status:** accepted  
- **Owner:** Signal team  
- **Related docs:**
  - `docs/plans/implementation.md`

---

## 1. Context

Signal is a real-time audio engine that must handle:

- IPC communication with Pulse over TCP
- Real-time audio processing with strict timing constraints
- State management (engine lifecycle, transport position)
- Graceful shutdown coordination

The audio processing thread has strict real-time constraints: it must never block, allocate memory, or perform I/O. All other operations (IPC, state updates, parsing) must be performed on non-real-time threads.

---

## 2. Problem Statement

How should Signal organise its threads to:

1. Maintain real-time guarantees for audio processing
2. Safely communicate between threads (IPC, audio, main)
3. Prevent data races and blocking operations in the audio thread
4. Support graceful shutdown

---

## 3. Options

### 3.1 Single-Threaded with Async I/O

**Description**

Use a single thread with async I/O (Asio) for all operations, including audio processing.

**Pros**

- Simple threading model
- No synchronization primitives needed
- Easy to reason about

**Cons**

- Cannot guarantee real-time audio processing
- I/O operations may block audio
- Violates real-time audio requirements

**Conclusion**

Rejected — does not meet real-time audio constraints

---

### 3.2 Dedicated Audio Thread with Message Passing

**Description**

- Main thread: Process startup, owns core objects, manages IPC server
- IPC thread (Asio event loop): Handles TCP connections, parses envelopes
- Audio thread: Dedicated high-priority thread for audio callbacks
- Communication via atomic flags and lock-free structures

**Pros**

- Clear separation of concerns
- Real-time audio thread can run independently
- Lock-free communication minimises contention
- Standard pattern for audio engines

**Cons**

- Requires careful synchronization
- More complex than single-threaded

**Conclusion**

Accepted — this is the chosen approach

---

## 4. Decision

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
   - Dispatches commands to domain handlers
   - Sends events back to Pulse
   - **Does NOT** mutate engine/transport state directly

3. **Audio Thread**
   - Dedicated thread for audio processing
   - High priority (platform-specific)
   - Processes audio buffers in real-time
   - Reads engine/transport state via lock-free atomics or snapshots
   - **Never** blocks, allocates, or performs I/O

### Communication Patterns

**IPC Thread → Main Thread**

- Commands are dispatched to domain handlers synchronously
- Domain handlers run on the IPC thread (Asio handler context)
- Domain handlers update `EngineHost` and `TransportState` objects
- These objects are accessed from the IPC thread only (no concurrent access)

**Main Thread → Audio Thread**

- Atomic flags: `std::atomic<bool>` for engine running state
- Lock-free state snapshots: Engine and transport state copied atomically
- No mutexes or locks in audio thread

**Audio Thread → Main Thread**

- Minimal stats via atomics: CPU load estimates, XRuns counters
- No complex data structures or allocations

### State Ownership

- `EngineHost`: Owned by main thread, accessed from IPC thread handlers
- `TransportState`: Owned by main thread, accessed from IPC thread handlers
- Audio thread reads state via atomic flags and lock-free snapshots

---

## 5. Rationale

1. **Real-time guarantees**: Dedicated audio thread with lock-free communication ensures audio processing never blocks

2. **Simplified synchronization**: IPC handlers run synchronously in Asio context, so no concurrent access to engine/transport state

3. **Future-proofing**: This model scales to multiple clients and more complex audio processing

4. **Standard pattern**: Matches common audio engine architectures (JUCE, PortAudio, etc.)

---

## 6. Consequences

### 6.1 Positive

- Clear separation between real-time and non-real-time code
- Real-time audio thread never blocks
- IPC operations cannot interfere with audio processing
- Shutdown can be coordinated cleanly

### 6.2 Negative / Trade-offs

- More complex than single-threaded approach
- Must be careful about state access patterns
- IPC handlers must be fast to avoid blocking the event loop

### 6.3 Mitigations

- Domain handlers keep state updates minimal
- Heavy work (if needed later) should be offloaded to worker threads
- Use lock-free data structures for audio thread communication

---

## 7. Follow-Up Actions

1. Document thread ownership in file headers
2. Ensure all audio thread code is lock-free
3. Add assertions/checks to prevent blocking operations in audio thread

---

## 8. Notes

- The current implementation uses Asio's async I/O, which runs handlers on the same thread as `io_context.run()` calls
- For multi-client support in the future, consider a separate thread pool for command processing
- Audio thread priority should be set to maximum on the target platform

