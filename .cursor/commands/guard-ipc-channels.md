# Task: Validate That Aura Never Talks Directly to Signal

You are operating in **validation mode**.  
Do **not** modify any source files.  
Your job is to **scan and report**.

## Architectural Rule (Must Hold)

From the current architecture:

- **Aura never connects directly to Signal.**
- **Pulse is the only client of Signal.**
- **Signal only communicates with Pulse.**
- High-rate streams (gesture, metering, analysis, etc.) follow this path:
  - Signal ⇄ Pulse (internal connection, binary or structured as appropriate)
  - Pulse ⇄ Aura (JSON IPC envelopes only)
- Aura’s world ends at **Pulse IPC**. Signal is an implementation detail behind Pulse.

Any code or documentation that implies otherwise is a violation.

---

## What To Scan For

Search the whole repository (source, tests, docs, comments, TODOs) for anything that suggests a direct Aura ↔ Signal relationship.

Be especially suspicious of:

- Mentions like:
  - "Aura → Signal"
  - "Aura to Signal"
  - "renderer connects to Signal"
  - "Electron connects to Signal"
  - "UI connects to Signal"
  - "binary channel from Aura to Signal"
  - "gesture stream directly to Aura from Signal"
  - "analysis stream directly to Aura"
  - "Signal sends X directly to Aura"
  - "Aura opens a socket to Signal"
- Any `signal` usage or imports in:
  - Electron main process code
  - preload scripts
  - renderer / Svelte / domains code
- Any TODOs or comments planning:
  - "add direct Signal connection from Aura"
  - "hook up Signal analyser socket here"
  - "gesture side-channel (Aura ↔ Signal)"

Also check the IPC and connection layers for anything suspicious:

- `src/main/**`
- `src/renderer/**`
- `src/shared/**`
- `docs/**` (architecture, specs, ADRs, etc.)

---

## Validation Rules

For each suspicious finding, classify it as **OK** or **Problem** using this rubric:

### ✅ OK

- Code or docs clearly say:
  - Pulse connects to Signal
  - Pulse receives analysis/gesture/metering streams from Signal
  - Pulse forwards JSON IPC to Aura
- Signal is only mentioned in the context of:
  - Pulse internals (spawning, connecting, supervising)
  - Chorus / Pulse / Signal architecture docs that **explicitly** keep Aura out of the link.

### ❌ Problem

- Any code where Aura / Electron main / renderer:
  - opens a socket to Signal
  - imports a `signal-*` client
  - holds a `SignalConnection` or similar type
  - reads/writes a Signal port
- Any docs / comments that:
  - describe or recommend Aura ↔ Signal direct channels
  - mention "side channel Aura → Signal"
  - suggest gesture/analyser streams going straight from Signal to Aura.

If in doubt, err on the side of **flagging** it as a Problem and explain why.

---

## Output Format

Create a **report file**:

- Path: `docs/reports/<timestamp>-signal-architecture-validation.md`
- Use the timestamp format: `YYYY-MM-DD-HHMMSS`, e.g. `2025-11-22-153045-signal-architecture-validation.md`.
- **Do not** modify or delete older report files.

The report must contain:

### 1. Summary

- A short paragraph summarising whether the repo currently respects the “Pulse-only access to Signal” rule.

### 2. Findings

For each issue or relevant mention, add a section:

- File path
- Line(s) or a very short excerpt (no need to quote full functions)
- Classification: `OK` or `Problem`
- Brief explanation

Example:

```md
## src/renderer/domains/transport.ts

- **Classification:** OK  
- **Reason:** Mentions transport state from Pulse only; no reference to Signal.

## src/main/signal-connection.ts

- **Classification:** Problem  
- **Reason:** Electron main opens a TCP socket directly to Signal and exposes it to renderer.
```

### 3. Recommendations

If you found **any** Problems:

- Suggest concrete, minimal changes to bring the code back into compliance, e.g.:
  - “Move this connection into Pulse”
  - “Rewrite this comment to reflect Pulse-only access”
  - “Replace this TODO with a Pulse-mediated design”

Do **not** apply those changes yourself in this command; just describe them.

---

## Important Constraints

- 🔒 **Do not modify any source files** in this task.
- 🔒 Do not rename, move, or delete files.
- ✅ You may read any file needed to make a confident assessment.
- ✅ The only new file you may create is the single report in `docs/reports/`.

Begin by scanning for all relevant mentions, then write the report.
