# g10.007 Plugin Domain Pruning To Real Foundations

Status: recorded
Owner: core-product
Date: 2026-06-11
Related roadmaps: `docs/roadmaps/g10/007-plugin-domain-pruning-to-real-foundations.md`

## Summary

Pruned the plugin domain to its real foundations: CLAP discovery FFI, VST3
factory introspection, AU plist scanning, LV2 manifest scanning, the
contracts vocabulary, block transport + watchdog core, the inventory data
model, and the sandbox process plumbing (spawn/stdio/shm). Deleted the
simulated lifecycle/process/state theatre, the fictional LV2 catalog, the
plugin-event continuity analyzers, the demo-flavor broker commands, the
in-process sandbox control envelopes, and the `signal-plugin-library` /
`signal-plugin-library-store` crates. Fixed the live discovery-safety bug:
host boot no longer scans the operator's real plugin directories, and CLAP
discovery no longer instantiates plugins unless explicitly opted in.

## Discovery Safety Fix

- `local_demo_runtime_assembly` (signal-host-local) no longer defaults
  `scan_roots` to `~/Library/Audio/Plug-Ins/CLAP`; the default is EMPTY and a
  sandbox is only assembled when the `SIGNAL_HOST_DEMO_PLUGIN_*` env override
  points at an explicit fixture root.
- VST3/AU/LV2 `discover_plugins_for_roots` no longer fall back to
  `default_scan_roots` (system directories) when the root list is empty;
  empty roots scan nothing. `default_scan_roots` remains as an explicit
  opt-in helper.
- CLAP discovery is factory-descriptor-only by default
  (`clap_entry` → factory → `get_plugin_count`/`get_plugin_descriptor`; no
  `create_plugin`). In-process capability probing (ports/params/extensions)
  is behind `ClapPluginHostAdapter::discover_plugins_for_roots_with_options
  (roots, probe_capabilities = true)` for trusted fixtures or a future
  sandboxed scanner. Descriptor-only scans report an all-zero I/O layout,
  empty buses/parameters, and a conservative state contract.
- Verified: `pulse cargo test --lib` in parallel passes with zero third-party
  plugin (Keepsake) spawns.

## Per-Crate LoC Deltas (non-blank Rust lines, incl. tests)

| crate | before | after |
| --- | --- | --- |
| signal-plugin | 3598 | 2994 |
| signal-plugin-clap | 4937 | 1285 |
| signal-plugin-au | 1378 | 785 |
| signal-plugin-lv2 | 1294 | 682 |
| signal-plugin-vst3 | 1990 | 1340 |
| signal-plugin-sandbox | 2464 | 463 |
| signal-plugin-inventory | 88 | 88 |
| signal-runtime | 59684 | 58313 |
| signal-host-local | 17533 | 7459 |
| signal-plugin-library(+store) | 384 | 0 (deleted) |

Workspace diff: 192 files, +802 / −20805 lines.

## Surviving Public API Inventory (plugin domain)

- `signal-plugin`: contracts vocabulary (`plugin_model`), events + codecs,
  block transport (`plugin_block_transport`), watchdog/sandbox policy types
  (`sandbox_protocol`: `PluginSandboxRequest`, `SandboxTransport`,
  `PluginSandboxCapabilities`, watchdog/escalation types,
  `PluginRenderContext`), `EventPacketSummary`/`ParameterAutomationSummary`.
- `signal-plugin-clap`: `ClapPluginHostAdapter` (descriptor-only discovery,
  opt-in capability probe), `ClapDiscoveredPluginType`, `ClapHostExtension`.
- `signal-plugin-vst3`: discovery (bundle scan + real COM
  `countClasses`/`getClassInfo` introspection + plist/moduleinfo parsing),
  `Vst3DiscoveredPluginType`, scan-root model.
- `signal-plugin-au`: plist discovery pre-filter, `AuDiscoveredPluginType`,
  scan-root model.
- `signal-plugin-lv2`: `manifest.ttl` bundle scanning with diagnostics only.
- `signal-plugin-sandbox`: broker shell with command set
  `status | attach | run | run-timeout | teardown | shutdown` — real child
  process, stdio receipts (wire format unchanged, plus `timed_out`), and
  verified file-backed shared-memory block round-trips. Documented as the
  real-hosting seed.
- `signal-runtime`: `SandboxBrokerClientSession` (spawn/timeouts/stderr
  drain), generic attach/run/run-timeout/teardown/shutdown client methods,
  `ensure_prepared_sandbox_session`, `teardown_broker_sandbox_session`.
- `signal-host-local`: explicit-roots scanning, metadata-only sandbox records
  on the direct path, broker-backed sessions when
  `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND` is set, honest boot (handshake →
  configure → graph → hardware → scan → optional fixture sandbox → start →
  8 engine blocks through the output pump).

## Deleted

- CLAP passthrough harness (`clap_sandbox_harness`), instance control
  surfaces, `ClapBlockProtocol`/transport/events/event-translation.
- VST3 fabricated session/state layer (`session.rs` `format!` blobs,
  `execute_block` digests) and session model types.
- AU fabricated session/state layer and failure-contract injection.
- LV2 fictional scaffold catalog plus session/extension-negotiation records.
- `plugin_event_reports` continuity analyzers and the runtime state mirrors
  (`runtime_plugin_event_state`, automation/timeline continuity fields).
- Broker demo flavors (`attach-demo`/`run-demo`/`run-timeout-demo`,
  AU/LV2/VST3 attach/stream/refresh/timeout choreography) and
  `run_vst3_broker_execution_sequence`.
- Host-local fault-injection boots, recovery overlap/teardown machinery,
  harness-driven runtime block cycle, and their test estate.
- In-process sandbox control envelopes (`SandboxControlCommand/Request/
  Response`) — the stdio+shm broker protocol in `signal-ipc` is the one
  protocol.
- `signal-plugin-library`, `signal-plugin-library-store` (zero consumers;
  Pulse owns the product plugin-library model).

## Validation

- `cargo build --workspace` clean (zero warnings).
- `cargo test --workspace -- --test-threads=1` green.
- pulse `cargo test --lib` (parallel) green, no Keepsake output.
- aura `cargo check` green.
- CLAP rustc-compiled fixture discovery tests green in both descriptor-only
  and probe modes.

## Left For Follow-Up (g10.009 candidates)

- Runtime LV2 prepared-negotiation observability
  (`RuntimeLv2PreparedNegotiationRecord`, lv2 extension snapshot surfaces)
  now has no producers; prune or rebuild with real LV2 hosting.
- Hardware device-loss recovery (simulation hooks and boots) was removed with
  the recovery theatre; real device-loss handling belongs to the rebuild
  program.
- `signal-runtime` plugin lifecycle/recall/transport-concurrency state models
  remain richer than what the pruned host exercises; revisit during the
  CLAP-first real-hosting program.
