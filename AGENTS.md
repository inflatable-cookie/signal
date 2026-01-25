# AGENTS

Guidance for AI and automated tools working in the **loophole-signal** repository.

Signal is a real-time audio engine implemented in C++20. It must be robust,
predictable, and maintainable. Prefer **foundational, durable solutions** over
quick hacks or speculative abstraction.

---

## btca

When you need up-to-date information about technologies used in this project, use btca to query source repositories directly.

**Available resources**: nlohmannJson, asio, clapSdk, miniaudio

### Usage

```bash
btca ask -r <resource> -q "<question>"
```

Use multiple `-r` flags to query multiple resources at once:

```bash
btca ask -r asio -r miniaudio -q "How should io threads be structured to avoid blocking audio processing?"
```

## Important: Significant Changes Require Reports

Any **significant** change made to the repository must include an accompanying changelog entry. The changelog entry must follow the project convention: create a new file in the **Chorus repository** at `loophole-chorus/reports/` named `<YYYY-MM-DD-HHMMSS>-<file-name>.md` describing the change.

**Old report files must never be modified.** Existing files in `docs/reports/` are historical artefacts and should be left unchanged.

## Important: Documentation Policy

All long-lived documentation (specs, ADRs, guides, references, reports) should be stored in the **Chorus** repository under `docs/`. Docs within Signal should generally be minimal stubs pointing to Chorus, unless there is a very strong repo-local reason.

When making significant changes or completing non-trivial tasks, write a **report** in the Chorus repo at:
- `loophole-chorus/reports/`

Use the filename format:
- `YYYY-MM-DD-HHMMSS-file-name.md`

**Important:** The timestamp `YYYY-MM-DD-HHMMSS` must be the **actual current date and time** (UTC) when creating the report, not a placeholder. Use a command like `date -u +"%Y-%m-%d-%H%M%S"` to generate the correct timestamp.

Do **not** modify the content of historical report files; always create a new report for new work.

---

## 1. Purpose of Signal

Signal is the **real-time audio engine** for Loophole:

- Receives commands/events from Pulse over Chorus IPC envelopes.
- Executes audio processing, plugin hosting, and hardware I/O.
- Must be robust and predictable; crashes here are bad but recoverable via Pulse.
- Operates with strict real-time safety guarantees.

Signal does not manage project data, UI state, or higher-level editing logic.
Those responsibilities belong to Pulse and Aura.

---

## 2. Language & Style

- Use **modern C++20**.
- Prefer RAII, smart pointers, and value types.
- Avoid raw `new`/`delete`; use `std::unique_ptr` / `std::shared_ptr` and standard containers.
- One major type per file (`.hpp` + `.cpp` pair).
- Keep functions small and focused.
- Use descriptive names; avoid opaque abbreviations.

### 2.1 Function Parameter Formatting

- Multi-line function parameters must be **one per line**, with the closing parenthesis aligned with the function keyword / name.
- No blank lines inside the parameter list.
- Functions with no parameters stay on a single line (no extra newlines).

```cpp
// ✅ Correct
void processEvent(
    const Envelope& envelope,
    DomainHandler& handler
) {
    // ...
}

// ✅ Correct – no params, single line
void reset() {
    // ...
}
```

### 2.2 Flow-Control Spacing

- Add a **blank line before and after** each flow statement (`if`, `while`, `for`, etc.) **inside a block**,  
  unless it is the **first or last statement** in that block.
- This visually separates control-flow from surrounding logic.

```cpp
void example() {
    initialise();

    if (condition) {
        handleCondition();
    }

    finalise();
}
```

### 2.3 Multi-line Boolean Conditions

- When a condition spans multiple lines, put logical operators (`&&`, `||`) at the **end of the current line**.
- Each clause should be on its own line.
- Only split onto multiple lines if there is **more than one clause**.
- For a single condition, keep it on one line.

```cpp
// ✅ Multi-clause: each clause on its own line
if (
    isReady && !isBlocked
    || hasOverride
) {
    // ...
}

// ✅ Single clause: no multiline
if (isReady) {
    // ...
}
```

---

## 3. IPC Consistency

- IPC envelope must match Chorus spec exactly:
  - `name` not `type`, no redundant domain prefixes, etc.
  - `kind` is one of: `command`, `event`, `snapshot`, `error`.
- Do not invent names or domains; follow Chorus docs.
- Put envelope struct and encoder/decoder in a dedicated `ipc` module.
- All IPC messages must use the Chorus envelope format defined in:
  - [`docs/specs/ipc/envelope.md`](https://github.com/infinite-loop-audio/loophole-chorus/blob/main/specs/ipc/envelope.md)

---

## 4. Project Layout

- `src/core` – core app types (e.g. `SignalApp`).
- `src/ipc` – envelope, codec, server/router, domain handler interfaces.
- `src/domains/<name>` – per-domain handlers (`engine`, `transport`, etc).
- `tests/` – C++ tests (unit + integration).
- `docs/` – minimal local docs (most documentation is in Chorus).

Keep modules small and focused. One major type per file.

---

## 5. Testing

- Use a single test framework (Catch2 for the skeleton).
- For any new behaviour, add or update tests.
- Keep tests close to the domain they cover, but under `tests/` (not mixed into `src/`).
- Tests should be clear and readable, following arrange–act–assert structure.

---

## 6. Canonical Pipelines & Dispatcher Pattern

- **Graph + schedule ingestion**: snapshots from Pulse are validated and applied once via `EngineDomain`/`EngineHost` into `GraphEngine` and `StreamScheduler`. No parallel ingestion paths or alternate snapshot formats.
- **Render path**: single render loop in `EngineHost` calling `GraphEngine::processBlock` with pre-allocated buffers; no legacy audio threads or duplicate render loops.
- **DomainDispatcher**: thin registry only; domains own their command handling. Do not add inline handlers or business logic to the dispatcher.
- **Real-time safety**: audio thread must not allocate, block, or log noisily. Prepare state and buffers on control threads; communicate via lock-free/atomic mechanisms.

---

## 7. Logging System

Signal uses a **unified logging system** that matches the same 1–8 log level scale as Aura and Pulse. All logs output to `stdout`/`stderr` with the standardised prefix format.

### 7.1 Log Levels

The unified log level system uses numeric levels 1–8 with threshold semantics (only logs where `log_level <= DEBUG_LEVEL` are emitted):

- **Level 1 – Core**: Absolute essentials (startup/shutdown, fatal errors)
- **Level 2 – Error**: Recoverable errors and serious failures
- **Level 3 – Warn**: Warnings indicating something is off but not fatal
- **Level 4 – Info**: Normal high-level operational info (default)
- **Level 5 – Debug**: Developer-oriented details
- **Level 6 – Verbose**: Detailed logs inside domains
- **Level 7 – Trace**: Very fine-grained events
- **Level 8 – All**: Everything including highly repetitive logs

### 7.2 Configuration

The `DEBUG_LEVEL` environment variable controls logging verbosity:

- Read from environment at startup (defaults to 4 if unset)
- Initialised via `initLogging()` (typically called in `SignalApp` constructor)
- Passed from Pulse when Signal is spawned as a child process

### 7.3 Usage

**Location**: `src/logging/Logging.hpp` and `Logging.cpp`

**Macros**:
```cpp
#include "logging/Logging.hpp"

// Single area
LOG_INFO({"SignalApp"}, "Initialising...");

// Multiple areas
LOG_DEBUG({"EngineHost", "Graph"}, "Graph has 5 nodes");

// Error logging
LOG_ERROR({"TcpServer"}, std::string("Failed to start: ") + error);

// With formatted messages
std::ostringstream msg;
msg << "Prepared asset: " << assetId;
LOG_INFO({"FileAudioAssetSource"}, msg.str());
```

**Format**: All logs follow `[Signal][LevelName][Area1][Area2] message...`

- Core and Error levels go to `stderr`
- All other levels go to `stdout`
- Aura's main process parses and colorises these logs

### 7.4 Available Macros

- `LOG_CORE(areas, message)` - Level 1 (Core)
- `LOG_ERROR(areas, message)` - Level 2 (Error)
- `LOG_WARN(areas, message)` - Level 3 (Warn)
- `LOG_INFO(areas, message)` - Level 4 (Info)
- `LOG_DEBUG(areas, message)` - Level 5 (Debug)
- `LOG_VERBOSE(areas, message)` - Level 6 (Verbose)
- `LOG_TRACE(areas, message)` - Level 7 (Trace)
- `LOG_ALL(areas, message)` - Level 8 (All)

**Macro syntax**: Areas use `std::initializer_list<std::string>`:
```cpp
LOG_INFO(({"EngineHost", "Transport"}), "Play command received");
```

**Note**: The double parentheses are required for the initializer list syntax in macros.

### 7.5 When to Use Each Level

- **Core (1)**: Critical lifecycle (Signal start, shutdown, fatal errors)
- **Error (2)**: Recoverable errors (plugin load failures, device errors)
- **Warn (3)**: Warnings (unknown node types, missing assets, invalid configurations)
- **Info (4)**: Normal operations (engine started, device selected, plugin scanned)
- **Debug (5)**: Detailed operations (graph building, schedule parsing, stream bindings)
- **Verbose (6)**: Extra context (detailed domain operations, audio processing details)
- **Trace (7)**: Per-envelope logging, fine-grained events
- **All (8)**: Everything including per-callback logging

### 7.6 Rules

- **Never use `std::cout`/`std::cerr` directly** except in the logging module implementation
- **Never use `printf` or other C-style output** — use the unified logging macros
- All logging must use the unified system for proper prefix formatting and `DEBUG_LEVEL` filtering
- Areas should be descriptive strings (e.g., `"SignalApp"`, `"EngineHost"`, `"GraphEngine"`)
- For multi-part areas, use multiple strings: `{"Domain", "SubComponent"}`
- Always pass message as `std::string` or use `std::ostringstream` for dynamic content
- **Real-time audio thread**: Logging from audio callbacks should use Trace/All levels only, and be minimal to avoid performance impact

### 7.7 Initialisation

Call `initLogging()` early in application startup (typically in `SignalApp` constructor):

```cpp
#include "logging/Logging.hpp"

SignalApp::SignalApp() {
    initLogging();  // Read DEBUG_LEVEL from environment
    // ... rest of initialisation
}
```

---

## 8. Real-Time Safety

Signal must adhere to real-time rules defined in Chorus:

- Real-time code must not:
  - Allocate or free memory
  - Acquire locks or use blocking operations
  - Perform I/O
  - Use dynamic container resizing
  - Execute unbounded computations

All JSON parsing and IPC handling must happen on non-real-time threads.

### 8.1 Audio Thread Rules

- The audio callback runs in a dedicated high-priority thread.
- The audio callback must never block, allocate, or perform I/O.
- Use lock-free data structures for communication between control and audio threads.
- State changes from IPC commands should be communicated via atomic flags or lock-free queues.

### 8.2 CPU Affinity

- Audio thread should ideally run on a dedicated CPU core (when available).
- Set audio thread priority to maximum (platform-specific).
- Keep control/IPC threads on separate cores to avoid audio thread interruptions.

### 8.3 Adding Future DSP Nodes

When adding DSP nodes to the audio graph:

- Pre-allocate all buffers at graph construction time.
- Never allocate in the audio callback.
- Use fixed-size buffers (no dynamic resizing).
- Validate all parameters before entering the audio thread.
- Keep node processing deterministic and bounded.

---

## 9. AI / Cursor Expectations

- Read existing docs before making changes.
- Keep changes small and scoped.
- Prefer refactors that **reduce** complexity and coupling.
- When updating IPC, keep Chorus, Signal, Pulse and Aura in sync.
- Do **not** modify old report files (e.g. `docs/reports/*` or timestamped reports).
- Always update `CHANGELOG.md` for significant changes.
- Write reports in the Chorus repository, not in Signal's local `docs/reports/`.

---

## 10. Build & Run

To configure and build:

```bash
cmake -S . -B build
cmake --build build
```

To run tests:

```bash
ctest --test-dir build
```

Or run the executable directly:

```bash
./build/loophole-signal
```

---

## 11. Changelog Discipline

You must **always** update `CHANGELOG.md` for any **significant** change:

- new feature or user-visible behaviour
- refactor that affects architecture or APIs
- bug fix or stability/shutdown fix
- documentation/spec changes
- build, tooling, or test infrastructure changes

Minor cosmetic edits or trivial renames **do not** need a changelog entry unless they impact behaviour or developer workflow.

### 9.1 File & section

- All entries go into the root `CHANGELOG.md`.
- New entries are added under the **`[Unreleased]`** heading.
- **Newest entries go at the top** of the `[Unreleased]` list.

### 9.2 Entry format

- Exactly **one line per change**, no multi-line blocks.
- Each line must include:
  - **UTC timestamp** in the form: `YYYY-MM-DD HH:MM:SS UTC`
  - An inline **tag** in square brackets, for example:
    - `[added]` – new features / new files
    - `[changed]` – behaviour changes, refactors, API tweaks
    - `[fixed]` – bug fixes, crashes, shutdown issues
    - `[removed]` – removals, deprecations
    - `[docs]` – documentation / spec changes
    - `[dev]` – tests, build system, CI, tooling
  - A **short, informative summary** in British English.
- Example:
  - `(2025-11-21 22:46:10 UTC) [added] Initial C++20 project skeleton with CMake build system and IPC envelope structure.`

### 9.3 Content style

- Do **not** copy-paste the full Cursor prompt into the changelog.
- Instead, write a concise summary of what actually changed and why it matters.
- Keep it **short but informative** (one sentence).
- Use British English spelling.

### 9.4 Discipline

- When you complete a significant change:
  - Finish your code / docs edits.
  - Then **immediately** add a new `[Unreleased]` entry to `CHANGELOG.md`.
- Never reorder or rewrite older entries.
- Never delete historical entries.
- Never remove the `[Unreleased]` section.

---

The default posture is: **clean, explicit, and careful**, not quick and dirty.
