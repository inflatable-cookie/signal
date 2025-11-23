# AGENTS

Guidance for AI and automated tools working in the **loophole-signal** repository.

Signal is a real-time audio engine implemented in C++20. It must be robust,
predictable, and maintainable. Prefer **foundational, durable solutions** over
quick hacks or speculative abstraction.

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
  - [`docs/specs/ipc/envelope.md`](https://github.com/infinite-loop-audio/loophole-chorus/blob/main/docs/specs/ipc/envelope.md)

---

## 4. Project Layout

- `src/core` – core app types (e.g. `SignalApp`).
- `src/ipc` – envelope, codec, server/router, domain handler interfaces.
- `src/domains/<name>` – per-domain handlers (`engine`, `transport`, etc).
- `tests/` – C++ tests (unit + integration).
- `docs/` – plans and architecture.

Keep modules small and focused. One major type per file.

---

## 5. Testing

- Use a single test framework (Catch2 for the skeleton).
- For any new behaviour, add or update tests.
- Keep tests close to the domain they cover, but under `tests/` (not mixed into `src/`).
- Tests should be clear and readable, following arrange–act–assert structure.

---

## 6. Real-Time Safety

Signal must adhere to real-time rules defined in Chorus:

- Real-time code must not:
  - Allocate or free memory
  - Acquire locks or use blocking operations
  - Perform I/O
  - Use dynamic container resizing
  - Execute unbounded computations

All JSON parsing and IPC handling must happen on non-real-time threads.

### 6.1 Audio Thread Rules

- The audio callback runs in a dedicated high-priority thread.
- The audio callback must never block, allocate, or perform I/O.
- Use lock-free data structures for communication between control and audio threads.
- State changes from IPC commands should be communicated via atomic flags or lock-free queues.

### 6.2 CPU Affinity

- Audio thread should ideally run on a dedicated CPU core (when available).
- Set audio thread priority to maximum (platform-specific).
- Keep control/IPC threads on separate cores to avoid audio thread interruptions.

### 6.3 Adding Future DSP Nodes

When adding DSP nodes to the audio graph:

- Pre-allocate all buffers at graph construction time.
- Never allocate in the audio callback.
- Use fixed-size buffers (no dynamic resizing).
- Validate all parameters before entering the audio thread.
- Keep node processing deterministic and bounded.

---

## 7. AI / Cursor Expectations

- Read existing docs before making changes.
- Keep changes small and scoped.
- Prefer refactors that **reduce** complexity and coupling.
- When updating IPC, keep Chorus, Signal, Pulse and Aura in sync.
- Do **not** modify old report files (e.g. `docs/reports/*` or timestamped reports).
- Always update `CHANGELOG.md` for significant changes.

---

## 8. Build & Run

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

## 9. Changelog Discipline

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

