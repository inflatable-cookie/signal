# 007 - Plugin Domain Pruning To Real Foundations

Status: complete
Owner: core-product
Created: 2026-06-11
Depends on: g10.001
Vision tags: `PLUGINS`, `DEMOLITION`, `FOUNDATIONS`

## Problem

No audio has ever passed through a third-party plugin in this codebase. Real
artifacts exist — CLAP discovery does genuine clap-sys/libloading FFI (its
tests compile a real CLAP cdylib with rustc), VST3 introspection does real COM
factory enumeration, the contracts vocabulary maps cleanly to CLAP, and the
block-transport/watchdog core is sound design. But the rest is simulation:
CLAP "process" copies input to output; VST3 fabricates state as format!
blobs; AU is plist parsing with zero AudioComponent FFI; LV2 resolves against
a hardcoded catalog of fictional plugins; the sandbox broker runs canned demo
flavors. signal-plugin-library duplicates Pulse's own plugin-library model;
signal-plugin-library-store has traits with zero implementations anywhere.

Pruning to the real foundations makes the eventual real-hosting program
(backlog: CLAP-first) start from truth instead of theatre.

Live evidence (2026-06-11): Pulse's `embedded_authority_can_capture*` tests
crash (SIGABRT/SIGTRAP) when run in parallel because host boot runs CLAP
discovery over the operator's real plugin directories and instantiates
third-party plugins (Keepsake wrapper) concurrently in-process. Discovery
must not instantiate arbitrary plugins in the control process, and tests
must not scan real plugin directories. Fix lands in this packet.

## Goals

- [x] keep and isolate: CLAP discovery FFI, VST3 factory introspection,
      contracts vocabulary, block transport + watchdog, inventory data model
- [x] delete simulated lifecycle/process/state surfaces: CLAP passthrough
      harness paths, VST3 fabricated sessions, AU and LV2 adapter fictions
      (retain plist/manifest scanning only as discovery pre-filters where
      real)
- [x] delete the sandbox broker's demo-flavor command set; keep the process
      spawn + stdio + shared-memory plumbing as the seed for real brokering
- [x] delete `signal-plugin-library` and `signal-plugin-library-store`
      (Pulse owns the product's plugin-library model)
- [x] delete plugin_event_reports continuity analyzers (premature
      observability for streams no plugin produces)
- [x] delete the parallel in-process "IPC protocol" envelope types or reduce
      them to the contracts crate — one protocol, defined where the broker is

## Non-Goals

- [ ] no real instance lifecycle / process() bridging / GUI embedding — that
      is the backlog rebuild program, pulled when Loophole schedules plugin
      hosting
- [ ] no new plugin formats

## Execution Plan

### Batch 7.1 - Library/Store And Reports

- [ ] confirm zero consumers (audit: Aura's plugin panel reads renderer
      fixtures, Pulse has its own model); delete both crates + event reports

### Batch 7.2 - Format Adapter Pruning

- [ ] CLAP: keep discovery; delete passthrough block surface and simulated
      control surfaces
- [ ] VST3: keep introspection; delete fabricated session/state layer
- [ ] AU/LV2: reduce to manifest/plist scan or delete outright (LV2 scaffold
      catalog dies)
- [ ] per-format build + test gate; real-FFI tests (CLAP fixture compile)
      stay green

### Batch 7.3 - Sandbox Honesty

- [ ] strip demo flavors from the broker binary; document the kept plumbing
      (spawn, stdio protocol, shm blocks) as the real-hosting seed
- [ ] align with the broker-session hardening in g10.005

## Acceptance Criteria

- [ ] every remaining public item in the plugin domain is either real FFI,
      the contracts model, transport/watchdog core, or inventory data
- [ ] CLAP discovery integration test (rustc-compiled fixture) green
- [ ] workspace + pulse + aura builds green

## Risks and Mitigations

- Risk: signal-runtime references deleted adapter surfaces.
- Mitigation: sequence after or alongside g10.005 batches; grep gates.

## Evidence Requirements

- [ ] per-crate LoC deltas; surviving-API inventory in the progress log

## Progress (2026-06-11)

- Landed in one commit, −20,810 LoC net across the domain. Per-crate:
  clap 4.9k→1.3k (sandbox harness/control surfaces/block protocol out;
  discovery FFI + catalog kept), vst3 2.0k→1.3k (fabricated sessions out;
  COM introspection kept), au 1.4k→0.8k (plist pre-filter kept), lv2
  1.3k→0.7k (fictional scaffold dead; manifest scanning only), plugin
  3.6k→3.0k (continuity analyzers + in-process envelope types out),
  sandbox 2.5k→0.5k (broker rewritten: status/attach/run/run-timeout/
  teardown/shutdown over real file-backed shm leases with verified block
  round-trips; wire format compatible with the hardened
  sandbox_broker_support), host-local 17.5k→7.5k (harness-driven
  boot/recovery theatre out; honest boot path), library/-store deleted.
- Live safety bug fixed both halves: discovery roots are explicit
  configuration defaulting EMPTY (system-dir fallback removed from all
  four adapters and the host-local demo assembly); CLAP discovery is
  factory-descriptor-only by default — instance probing behind an explicit
  probe_capabilities opt-in. Proof: pulse `cargo test --lib` PARALLEL
  passes 122/122 with zero keepsake spawns (previously SIGABRT).
- Evidence log: docs/logs/2026-06/11-144500-g10-007-plugin-domain-pruning-
  to-real-foundations.md. Left for later: lv2 prepared-negotiation
  observability has zero producers; richer runtime plugin state models
  revisit during the CLAP-first rebuild program.

## Next Task

g10.008 (DSP corrections and resampling) — parallel lane.
