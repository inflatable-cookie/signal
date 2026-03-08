# 002 Supervisor Export Schema And Report Boundary

Status: active
Owner: core-product
Updated: 2026-03-08
Related architecture: `docs/architecture/system-architecture.md`
Related package map: `docs/architecture/package-map.md`

## Purpose

Freeze the boundary between the shared runtime supervisor report surface and
the versioned CLI export produced by `signal-supervisor-tools`, so automation
and host tooling consume a stable contract instead of scraping ad hoc output.

## Contract

1. The machine-readable supervisor export envelope uses:
   - `schema = "signal.supervisor.export"`
   - `schema_version = 1`
2. The export envelope contains exactly four top-level payload areas:
   - `profile`
   - `scenario`
   - `host_summary`
   - `supervisor_report`
3. `supervisor_report` is the shared authority surface for runtime-level
   diagnostics, supervision state, timeline continuity, and any continuity data
   promoted into `signal-runtime`.
4. `host_summary` is an assembly-level supplement for profile-specific or
   profile-shaped fields that are not part of the shared runtime report.
5. `host_summary` should not mirror runtime-owned readiness, diagnostics, or
   supervision state; those belong in `supervisor_report`.
6. Timeline continuity belongs in `supervisor_report`, not only in
   host-specific summaries.
7. Automation continuity belongs in `supervisor_report` through the shared
   `RuntimeAutomationSnapshot` surface.
8. `host_summary` should not mirror runtime-owned automation continuity fields.
   If a continuity field is needed outside `supervisor_report`, it must be
   justified as an assembly-local convenience rather than copied by default.
9. `host_summary` should not mirror runtime-owned block-sequence continuity
   fields either; sequence segments, epochs, gaps, and rollover counts belong
   in `supervisor_report`.
10. `host_summary` should not mirror runtime-owned automation counters or
    automation value snapshots either; those belong in
    `supervisor_report.automation`.
11. Assembly-local payload outcomes may remain in internal host summaries for
    tests and debugging, but they are not part of the default exported
    `host_summary` contract in schema version 1.
12. `signal-supervisor-tools` may expose payload detail only through an
    explicit opt-in debug path such as `--include-payload`; that opt-in adds a
    grouped `payload` block without changing the meaning of the default export.
13. Assembly-local control, transport, and fault detail should follow the same
    rule: grouped host-local execution blocks are preferred over flat summary
    fields when those details do not belong in `supervisor_report`.
14. `host_summary` should declare which grouped sections are present through a
    stable `sections` list so automation can distinguish default and opt-in
    debug exports without inferring intent from missing keys alone.
15. `host_summary` should also declare `debug_sections_supported` and
    `debug_sections_enabled` so the current targeted debug policy is visible in
    the export itself, not only in documentation.
16. The preferred grouped `host_summary` shape is:
   - top-level identity/profile fields only
   - `sections`
   - `debug_sections_supported`
   - `debug_sections_enabled`
   - `execution`
   - `transport`
   - `faults`
17. When payload detail is explicitly requested, it should appear as one
    grouped `payload` block rather than as top-level counter sprawl.
18. The current targeted debug-section model supports only `payload`; adding
    any new opt-in section requires an explicit contract and implementation
    batch rather than silent flag growth.
19. Schema evolution must be deliberate:
   - additive fields may extend `schema_version = 1` if existing fields keep
     their meaning
   - breaking shape changes require a new `schema_version`
20. `signal-supervisor-tools --describe-export` is the canonical host-free
    introspection path for the frozen schema version, default host-summary
    sections, and supported debug sections.

## Placement Decision

- Block-sequence continuity is runtime-owned and belongs directly in
  `supervisor_report`.
- Automation continuity is now also runtime-owned and belongs in
  `supervisor_report` through `RuntimeAutomationSnapshot`.

## Acceptance Signals

- `signal-supervisor-tools --format=json ...` emits the versioned envelope.
- `signal-supervisor-tools --format=json ...` emits grouped `host_summary`
  blocks instead of a flat execution-field dump.
- `signal-supervisor-tools --format=json ...` exposes the default grouped
  section list through `host_summary.sections`.
- `signal-supervisor-tools --format=json ...` exposes the current debug policy
  through `host_summary.debug_sections_supported` and
  `host_summary.debug_sections_enabled`.
- `signal-supervisor-tools --describe-export --format=json` exposes the frozen
  schema/version and supported debug-section policy without running a host.
- `signal-supervisor-tools --format=json ...` does not export host-local
  payload detail by default.
- `signal-supervisor-tools --format=json --include-payload ...` may add one
  grouped `payload` block for explicit debugging, and `host_summary.sections`
  expands accordingly.
- no other opt-in debug section is currently supported.
- `RuntimeSupervisorReport` exposes timeline continuity directly.
- `RuntimeSupervisorReport` exposes automation continuity directly through
  `RuntimeAutomationSnapshot`.
- Docs point to this contract when describing supervisor export behavior.

## Next Task

Move away from supervisor-export policy work and pick the next central engine
slice, most likely runtime/host control-path hardening or the next real
plugin-sandbox lifecycle increment.
