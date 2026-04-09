# 002 - Shared Plugin-Hosting Substrate And Hardened Sandbox Execution

Status: complete
Owner: core-product
Created: 2026-04-08
Depends on: g09.001
Vision tags: `PLUGIN`, `SANDBOX`, `RUNTIME`
Contract refs: `007`, `008`, `014`, `072`

## Problem

Signal's plugin stack is still scaffolded: discovery paths are fixture-backed,
the sandbox binary is a demo harness, and host integration does not yet prove
one real cross-format scan/load/process substrate.

## Goals

- [ ] build one real shared plugin-hosting foundation for VST3, AU, and LV2
- [ ] replace the demo sandbox process with a hardened long-lived broker
- [ ] keep runtime-owned lifecycle, continuity, and fault meaning intact

## Non-Goals

- [ ] no editor UI embedding or product-local browser shell
- [ ] no vendor certification matrix in this milestone

## Execution Plan

### Batch 2.1 - Discovery And Capability Matrix Freeze

- [x] inventory every current fixture-backed discovery and synthetic capability
      path across `signal-plugin-*`, `signal-runtime`, and host crates
- [x] define the real scan-root, disabled, unsupported, and unavailable states
      that runtime receipts must expose
- [x] add one shared adapter-capability matrix covering scan, load, instantiate,
      process, state, and sandbox support per format and platform

### Batch 2.2 - Real Discovery Foundation

- [x] replace fixture-backed discovery front doors with filesystem-backed scan
      entry points in each adapter crate
- [x] standardize module, bundle, manifest, and component provenance capture in
      runtime-owned discovery receipts
- [x] keep test fixtures as test-only helpers instead of production code paths

### Batch 2.3 - Hardened Sandbox Process

- [x] replace the `signal-plugin-sandbox` synthetic lifecycle loop with a real
      request-serving process boundary
- [x] define startup, ready, attach, running, teardown, crash, and timed-out
      lifecycle states as typed sandbox receipts
- [x] harden shared-memory lease, cleanup, and ownership handoff hooks in the
      shared sandbox path

### Batch 2.4 - Cross-Host Integration Proof

- [x] route host-local and host-server through the same shared sandbox process
      contract
- [x] add focused scan/load/process smoke lanes for each host surface
- [x] expose explicit "not supported here yet" answers for gaps that remain
      deferred after the foundation lands

## Acceptance Criteria

- [x] discovery comes from real adapter scan roots rather than canned fixture
      output
- [x] the sandbox process is a real broker with typed lifecycle and failure
      behavior
- [x] local and server hosts consume the same runtime-owned plugin substrate

## Risks And Mitigations

- Risk: format-specific details leak into one shared scanner abstraction too
  early.
- Mitigation: standardize only the runtime receipt seam and keep traversal
  detail adapter-local.

- Risk: sandbox hardening regresses current CLAP and runtime continuity paths.
- Mitigation: land the broker state machine first, then migrate formats one by
  one behind focused smoke lanes.

## Evidence Requirements

- [ ] log each discovery and sandbox tranche
- [ ] run `cargo check -p signal-plugin-sandbox`
- [ ] run `cargo check -p signal-runtime`
- [ ] run `effigy health`

## Batch 2.1 Outcome

Batch 2.1 is now frozen in the planning surface and contract layer instead of
living only in the audit notes.

The current-state inventory now records the real shared foundation problem:

- `signal-plugin-vst3`, `signal-plugin-au`, and `signal-plugin-lv2` all expose
  platform scan roots, but discovery still synthesizes plugin records from
  `src/fixtures.rs`
- both host crates still route scan requests through demo/discovery helpers,
  then feed those bounded results into runtime-owned plugin receipts
- `signal-plugin-sandbox` is still a synthetic CLAP lifecycle shell instead of
  a long-lived request-serving broker

The shared capability matrix and rules are now frozen in
`docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`,
including the current implementation posture and the rule that demo assemblies
cannot remain the production authority once real discovery and execution land.

The runtime-facing discovery states needed for later implementation are now
explicit for this queue:

- real scan roots
- disabled roots
- unsupported platform or format paths
- unavailable or malformed discovery paths

That gives Batch 2.2 one concrete target: replace fixture discovery without
changing the runtime-owned receipt boundary again.

## Batch 2.2 Outcome

Batch 2.2 is now complete.

The adapter scan path no longer synthesizes discovered plugins straight from
fixture ids inside `discover_plugins_for_roots(...)`:

- `signal-plugin-vst3` now walks real `.vst3` entries under the requested scan
  roots and records the actual `module_root`
- `signal-plugin-au` now walks real `.component` entries under the requested
  scan roots and records the actual `bundle_root`
- `signal-plugin-lv2` now walks real `.lv2` bundle entries under the requested
  scan roots and records the actual `bundle_root` plus manifest path

That means host-local and host-server scan requests now feed runtime-owned
plugin discovery receipts from real filesystem roots instead of from the old
root-match plus fixture-id shortcut.

Batch 2.2 now covers the full shared discovery foundation:

- adapter discovery walks real filesystem roots for VST3, AU, and LV2
- runtime-owned receipts carry real module, bundle, and manifest provenance
- host-local and host-server sandbox ensure reuse discoveries cached from scan
  time instead of rediscovering by fixture id
- affected host proof surfaces now create explicit temporary scan roots
- AU, VST3, and LV2 adapter discovery metadata now lives in production-owned
  scaffold modules under each adapter instead of crate-level test fixture
  modules

## Batch 2.2 Tranche 2 Outcome

This tranche finished the shared host-side discovery handoff that Batch 2.2
needed before sandbox hardening can start.

The production host path changed in two important ways:

- host-local and host-server now cache AU, VST3, and LV2 adapter discoveries
  from `start_plugin_scan(...)`
- sandbox ensure now instantiates from those cached discoveries instead of
  asking adapters to rediscover by plugin-type id during ensure

The proof surfaces moved with it:

- host-internal plugin scan tests now create explicit temp plugin roots
- public VST3, AU, LV2, cross-adapter parity, generic event, and LV2 extension
  tests now create explicit temp roots instead of assuming user or system
  plugin directories

That closes the remaining production dependency on crate-level fixture modules
for discovery. The remaining major scaffold in this queue is now the synthetic
`signal-plugin-sandbox` process, which belongs to Batch 2.3.

## Batch 2.2 Tranche 3 Outcome

This tranche removed the last adapter-local production dependency on
`src/fixtures.rs`.

The AU, VST3, and LV2 crates now expose production-owned scaffold metadata
inside their adapter module trees:

- `signal-plugin-vst3/src/vst3_host_adapter/scaffold.rs`
- `signal-plugin-au/src/au_host_adapter/scaffold.rs`
- `signal-plugin-lv2/src/lv2_host_adapter/scaffold.rs`

The old crate-level fixture modules were removed, and discovery now matches
real filesystem entries against scaffold bundle names and scaffold discovered
records instead of importing test-fixture modules.

Focused validation passed for:

- adapter unit tests in `signal-plugin-vst3`, `signal-plugin-au`, and
  `signal-plugin-lv2`
- host compile checks in `signal-host-local` and `signal-host-server`
- targeted public VST3 proof tests on both hosts
- `effigy health`

## Batch 2.3 Tranche 1 Outcome

Batch 2.3 is now started in production code.

`signal-plugin-sandbox` no longer executes a one-shot synthetic lifecycle and
prints a single summary line from `main.rs`. It now hosts a long-lived broker
loop backed by typed broker receipts in `src/broker.rs`.

The new broker process currently supports:

- persistent stdin-driven command serving
- typed startup and ready receipts
- typed attached, running, teardown-complete, timed-out, crashed, and shutdown
  receipts
- a real demo execution path that drives the CLAP lifecycle harness, heartbeat,
  block processing, teardown sequence, and transport cleanup through one
  request-serving process boundary

This tranche intentionally stops short of full shared-memory lease hardening and
host adoption. The broker loop exists, the receipt vocabulary exists, and the
transport cleanup path now runs on timeout and shutdown, but the hosts are not
yet speaking to this broker as their primary sandbox boundary.

Focused validation passed for:

- `cargo check -p signal-plugin-sandbox`
- `cargo test -p signal-plugin-sandbox`
- `effigy health`

## Batch 2.3 Tranche 2 Outcome

This tranche moved Batch 2.3 from broker-local proof into one real host-owned
integration path.

The sandbox broker itself is now capable of holding a live attached session
instead of only running one-shot demos:

- `signal-plugin-sandbox` now supports `attach-demo` and `teardown-demo`
  commands in addition to `run-demo`, `run-timeout-demo`, `status`, and
  `shutdown`
- attach receipts now expose explicit lease ownership data before teardown
- teardown receipts now expose explicit cleanup outcomes for both normal and
  timeout paths

The host side now consumes that process boundary in one bounded production path:

- `signal-host-local` VST3 sandbox ensure can opt into the broker process when
  `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND` is configured
- that path records runtime-owned prepared-state, attach, detach, and instance
  destruction receipts from the broker lifecycle instead of from an in-process
  prepare session
- host-local teardown now drives broker teardown and shutdown, then records
  detached transport and torn-down lifecycle state back into runtime

The proof surface moved with it:

- `signal-host-local/tests/public_host_edge_sandbox_broker.rs` now proves the
  raw broker receipt protocol
- the same test surface now proves one real host-local VST3 scan,
  ensure-through-broker, and teardown-through-broker roundtrip

This tranche intentionally does not mark Batch 2.3 complete. The remaining
open work is still the broad shared-memory ownership and cleanup handoff for
both hosts, not just the first host-local VST3 path.

Focused validation passed for:

- `cargo test -p signal-plugin-sandbox`
- `cargo check -p signal-host-local`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `effigy health`

## Batch 2.3 Tranche 3 Outcome

This tranche widened the same broker-backed sandbox contract into the server
host instead of creating a second special-case protocol.

`signal-host-server` now mirrors the first bounded host-local adoption:

- server-side VST3 sandbox ensure can opt into the typed broker process when
  `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND` is configured
- server-side teardown now drives broker teardown and shutdown, then records
  detached transport, transport-torn-down, and instance-destroyed lifecycle
  receipts back into runtime
- both hosts now keep live broker-backed sessions in host state instead of
  treating the broker as a test-only subprocess

The proof surface now covers both hosts:

- `signal-host-local/tests/public_host_edge_sandbox_broker.rs` proves the local
  host VST3 broker attach and teardown path
- `signal-host-server/tests/public_host_edge_sandbox_broker.rs` proves the
  matching server host VST3 broker attach and teardown path

This still does not complete Batch 2.3 or Batch 2.4. The broker contract is
now present in both hosts, but only for the bounded VST3 branch guarded by the
broker environment, and the wider detach-fault / transport-cleanup paths still
live in duplicated host-local and host-server helpers.

Focused validation passed for:

- `cargo check -p signal-host-server`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`

## Batch 2.3 Tranche 4 Outcome

This tranche pulled the new broker-backed host behavior into a shared
host/runtime helper surface instead of leaving the same process and receipt
logic duplicated in both hosts.

The consolidation now lives in `signal-runtime`:

- `signal-runtime/src/sandbox_broker_support.rs` now owns the broker client
  process wrapper
- the same module now owns broker receipt parsing plus the prepared-session and
  detached-session runtime recording helpers

That removed the largest duplicated slab from the host crates:

- `signal-host-local/src/host_support/sandbox_sessions.rs` now mainly chooses
  local AU/VST3 instance ids and plugin-specific metadata
- `signal-host-server/src/host_support/sandbox_sessions.rs` now mainly chooses
  server AU/LV2/VST3 instance ids and plugin-specific metadata
- both hosts now delegate broker process spawning, startup receipt validation,
  attach, teardown, shutdown, and runtime receipt mapping to the shared runtime
  helper layer

This is still not the end of Batch 2.3. The broker-backed path remains bounded
to the VST3 branch, and the existing recovery cleanup helpers still do not use
the same shared broker surface for detach-fault and ownership handoff work.

Focused validation passed for:

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Batch 2.3 Tranche 5 Outcome

This tranche spent the shared broker/runtime layer on the first real recovery
cleanup seam instead of another steady-state host entrypoint.

The shared helper surface in `signal-runtime/src/sandbox_broker_support.rs`
now owns detach transition recording as well as steady-state broker attach:

- detach-requested recording is shared
- detach-fault recording is shared
- detached plus transport-torn-down recording is shared, with explicit control
  over whether the path should also mark instance destruction

That shared cleanup recording now drives the duplicated lingering transport
cleanup paths in both hosts:

- `signal-host-local/src/host_support/recovery_cleanup_transport.rs`
- `signal-host-server/src/host_support/recovery_cleanup_transport.rs`

Those files still own host-local broker destroy calls and CLAP teardown calls,
but they no longer hand-roll the runtime-owned detach state transitions. This
is the first place where the shared broker layer now covers ownership-handoff
and detach-fault behavior instead of only the steady-state VST3
ensure/teardown path.

The tranche also hardened the local broker public proof against test-process
environment races by serializing broker environment setup inside the support
helper.

Focused validation passed for:

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Batch 2.3 Tranche 6 Outcome

This tranche pushed the shared broker/runtime transition layer into the first
full overlap-recovery episode instead of stopping at cleanup tails.

The duplicated old-transport teardown path inside overlap recovery now routes
through the shared transition helpers in `signal-runtime/src/sandbox_broker_support.rs`:

- detach-requested recording during overlap finish is shared
- detach-fault recording during deferred teardown, destroy failure, injected
  old-transport teardown failure, and CLAP transport teardown failure is shared
- detached plus transport-torn-down recording after successful overlap cleanup
  is shared

That path now applies in both hosts:

- `signal-host-local/src/host_support/recovery_overlap_finish.rs`
- `signal-host-server/src/host_support/recovery_overlap_finish.rs`

This is the first place where the shared broker/runtime helper layer now owns a
full recovery-episode ownership transition instead of only steady-state sandbox
teardown and lingering cleanup tails.

Focused validation passed for:

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

Focused overlap lib-test execution is still blocked by pre-existing unresolved
split-test module paths in the host lib-test trees, which were already present
before this tranche and remain outside the scope of `g09.002`.

## Batch 2.3 Tranche 7 Outcome

This tranche widened the same shared broker/runtime ownership layer into the
replacement-rollback and origin-abort teardown path instead of leaving those
detach transitions duplicated in host-local and host-server recovery teardown.

The shared helper surface in `signal-runtime/src/sandbox_broker_support.rs`
now owns two higher-level teardown outcomes:

- broker detach failure recording now couples broker-failure and detach-fault
  emission in one runtime-owned helper
- successful detach completion now couples detached-plus-transport-torn-down
  recording with transport-session closure in one runtime-owned helper

Those higher-level teardown outcomes now drive duplicated recovery transport
cleanup and recovery teardown paths in both hosts:

- `signal-host-local/src/host_support/recovery_cleanup_transport.rs`
- `signal-host-server/src/host_support/recovery_cleanup_transport.rs`
- `signal-host-local/src/host_support/recovery_teardown.rs`
- `signal-host-server/src/host_support/recovery_teardown.rs`

This means the shared broker/runtime layer now covers:

- steady-state broker attach and teardown
- lingering cleanup detach-requested and detach-fault transitions
- overlap-finish ownership handoff and teardown transitions
- replacement rollback and origin-abort transport teardown outcomes

Batch 2.3 still remains open. The shared layer now owns more of the recovery
transport lifecycle, but the actual broker-backed execution path is still a
bounded VST3 branch and earlier recovery admission/start paths still retain
host-specific control flow.

Focused validation passed for:

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Batch 2.3 Tranche 8 Outcome

This tranche widened the broker-backed execution lane beyond the bounded
VST3-only path by routing AU sandbox ensure and teardown through the same
broker process contract in both hosts.

The host-side change is now format-broader instead of only recovery-deeper:

- `signal-host-local/src/host_support/sandbox_sessions.rs` now routes AU and
  VST3 through one broker attach helper when the broker environment is enabled
- `signal-host-server/src/host_support/sandbox_sessions.rs` now does the same
  for AU and VST3
- both hosts now retain AU broker sessions in host state instead of dropping
  the broker-backed ensure result before teardown

The proof surface widened with it:

- `signal-host-local/tests/public_host_edge_sandbox_broker.rs` now proves
  broker-backed AU and VST3 roundtrips
- `signal-host-server/tests/public_host_edge_sandbox_broker.rs` now proves the
  same broker-backed AU and VST3 roundtrips
- the server broker test support now serializes broker environment mutation the
  same way the local support helper already did

This does not complete Batch 2.3, but it materially changes the state of the
queue: the broker-backed execution path is no longer limited to VST3, and the
shared broker lane now covers the two plugin formats that both hosts already
implement.

Focused validation passed for:

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Batch 2.3 Tranche 9 Outcome

This tranche finished the multi-format widening pass on the server side by
bringing LV2 into the same broker-backed execution lane as AU and VST3.

The implementation change is intentionally narrow and structural:

- `signal-host-server/src/host_support/sandbox_sessions.rs` now routes LV2
  through the shared broker attach helper when the broker environment is
  enabled
- `signal-host-server/src/host.rs` now retains broker-backed LV2 sessions in
  host state so teardown drives the same typed broker teardown path instead of
  dropping back to generic sandbox teardown bookkeeping

The proof surface widened with it:

- `signal-host-server/tests/public_host_edge_sandbox_broker.rs` now proves
  broker-backed LV2 roundtrips alongside the existing AU and VST3 proofs

This materially changes the state of Batch 2.3:

- the server host broker-backed execution lane now covers AU, VST3, and LV2
- the local host broker-backed execution lane now covers AU and VST3
- the remaining open work is no longer “VST3-only broker support”, but the
  deeper unification of recovery/control flow and any remaining format gaps

Focused validation passed for:

- `cargo check -p signal-host-server`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Batch 2.3 Tranche 10 Outcome

This tranche finally pushed the shared runtime-owned broker layer into earlier
recovery state handling instead of only steady-state and teardown paths.

The new shared helper surface in `signal-runtime/src/sandbox_broker_support.rs`
now owns the basic recovery-overlap state transitions:

- entering overlap now uses one runtime-owned helper instead of direct
  host-local and host-server counter mutation
- collapsing overlap back to zero on rollback now uses one shared helper
- completing a successful overlap restart now uses one shared helper that
  restores the active sandbox count and promotes the recovered transport
  session back to steady state when one exists

Those earlier recovery-state helpers now drive:

- `signal-host-local/src/host_support/recovery_overlap_prepare.rs`
- `signal-host-server/src/host_support/recovery_overlap_prepare.rs`
- `signal-host-local/src/host_support/recovery_overlap_restart.rs`
- `signal-host-server/src/host_support/recovery_overlap_restart.rs`
- `signal-host-local/src/host_support/recovery_runtime.rs`
- `signal-host-server/src/host_support/recovery_runtime.rs`

This also closes one meaningful asymmetry: the server recovery restart path now
promotes recovered transport sessions back to steady state the same way the
local host already did.

Batch 2.3 still remains open, but the queue is in a better place now:

- shared broker ownership covers steady-state broker attach and teardown
- shared helpers cover lingering cleanup, overlap-finish teardown, rollback
  teardown, and earlier overlap-state transitions
- the remaining host-specific surface is increasingly about recovery control
  choreography rather than low-level broker/session truth

Focused validation passed for:

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Batch 2.3 Tranche 11 Outcome

This tranche pulled another duplicated early-recovery slab into the shared
runtime-owned broker helper surface: the recovery-cycle and invalidation setup
that runs before the hosts decide between lingering recovery and overlap
replacement.

The new shared helper in `signal-runtime/src/sandbox_broker_support.rs` now
owns the runtime-facing part of beginning a brokered recovery cycle:

- recording the recovery cycle with degraded-mode stop reason
- invalidating the active completion slot and lease epoch through the supplied
  lifecycle callback
- recording completion-slot invalidation and broker invalidation receipts with
  the correct recovery reason text

That shared helper now drives both `recover_sandbox(...)` entrypoints:

- `signal-host-local/src/host_support/recovery_sandbox.rs`
- `signal-host-server/src/host_support/recovery_sandbox.rs`

This is meaningful because the hosts now diverge later in the flow, after the
runtime-owned recovery bookkeeping is already normalized, instead of duplicating
that invalidation and receipt logic themselves.

Focused validation passed for:

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Batch 2.3 Tranche 12 Outcome

This tranche moved the remaining duplicated overlap contention and
replacement-start rollback interpretation out of the host recovery branches and
into the shared runtime-owned broker helper surface.

The shared helper layer in `signal-runtime/src/sandbox_broker_support.rs` now
owns the common outcome logic for:

- competing overlap-prepare attach attempts
- the synthetic "expected contention but attach succeeded" failure case
- replacement restart failure vs injected replacement-start failure vs runtime
  restart failure sequencing

Both hosts now use that shared sequencing surface in:

- `signal-host-local/src/host_support/recovery_overlap_prepare.rs`
- `signal-host-server/src/host_support/recovery_overlap_prepare.rs`
- `signal-host-local/src/host_support/recovery_overlap_restart.rs`
- `signal-host-server/src/host_support/recovery_overlap_restart.rs`

The host crates still perform the host-specific lifecycle work locally, but the
shared helper layer now decides how those outcomes map onto overlap rollback
and restart success instead of each host carrying its own copy of that
decision-making.

That is the first tranche in this queue where the shared broker/runtime layer
owns both the runtime-facing recovery state transitions and the interpretation
of the main overlap contention and replacement-start branches. The remaining
host-specific recovery work is now narrower and mostly about the last
format-independent rollback entrypoints that still wrap those helpers.

Focused validation passed for:

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Batch 2.3 Tranche 13 Outcome

This tranche moved another higher-level rollback wrapper seam out of the host
crates' duplicated detach bookkeeping and into the shared runtime-owned broker
helper surface.

The new shared helper in `signal-runtime/src/sandbox_broker_support.rs` now
owns the runtime-facing outcome mapping for brokered recovery transport detach:

- recording detach-requested state
- recording destroy-region detach failures
- recording teardown-active-transport detach failures
- completing a successful detach and transport-session end when neither fault
  path occurs

Both hosts now collapse their abort-origin and replacement-rollback teardown
entrypoints onto one local wrapper plus that shared helper:

- `signal-host-local/src/host_support/recovery_teardown.rs`
- `signal-host-server/src/host_support/recovery_teardown.rs`

That means the remaining host-specific logic in those entrypoints is now just:

- running the format-local lifecycle teardown sequence
- deciding whether the origin sandbox should be torn down first
- supplying the broker destroy and transport teardown outcomes back to the
  shared runtime-owned detach helper

This is another meaningful reduction because the hosts no longer duplicate the
detach-requested, failure-stage, and successful detach bookkeeping themselves
for both abort-origin and replacement-rollback teardown paths.

Focused validation passed for:

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Batch 2.3 Tranche 14 Outcome

This tranche moved the lingering-session restart wrapper out of duplicated host
control flow and into the shared runtime-owned broker helper surface.

The new shared helper in `signal-runtime/src/sandbox_broker_support.rs` now
owns the runtime-facing sequencing for lingering-session restart:

- interpreting plugin sandbox restart failure
- normalizing the pre-start overlap promotion back to one active sandbox
- promoting the recovered transport session to steady state when restart and
  lifecycle re-entry succeed
- rolling overlap state back to zero when the late runtime start fails

Both hosts now delegate that shared sequencing from:

- `signal-host-local/src/host_support/recovery_runtime.rs`
- `signal-host-server/src/host_support/recovery_runtime.rs`

The host crates still own the host-specific pieces:

- lingering origin cleanup
- orphan lingering cleanup before restart
- re-running the lifecycle to build the replacement session
- performing rollback teardown if the shared helper reports a restart or
  late-start failure

This is meaningful because the hosts no longer duplicate the same restart,
overlap-promotion, transport-promotion, and late-start rollback state logic in
parallel lingering-session recovery paths.

Focused validation passed for:

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Batch 2.3 Tranche 15 Outcome

This tranche pulled the old-transport teardown fault mapping in
`recovery_overlap_finish` into the shared runtime-owned broker helper surface.

The new shared helper in `signal-runtime/src/sandbox_broker_support.rs` now
owns the runtime-facing interpretation for overlap old-transport teardown:

- recording detach-requested state for the retiring transport
- handling deferred old-transport teardown failure without ending the
  transport session yet
- handling destroy-region and transport-teardown failures after transport
  ownership has already shifted
- recording successful detach and ending the retired transport session when
  teardown completes cleanly

Both hosts now delegate that shared outcome mapping from:

- `signal-host-local/src/host_support/recovery_overlap_finish.rs`
- `signal-host-server/src/host_support/recovery_overlap_finish.rs`

The host crates still own the host-specific lifecycle work and rollback branch
selection around that helper, but they no longer duplicate the runtime-facing
detach bookkeeping and failure-stage mapping for the old transport.

This materially tightens the remaining Batch 2.3 scope: most of the recovery
ownership and transport truth is now shared, and the remaining open work is
less about duplicated recovery control flow and more about broadening proof and
format coverage around the shared broker process contract.

Focused validation passed for:

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Batch 2.3 Tranche 16 Outcome

This tranche shifted Batch 2.3 from helper extraction into real broker-backed
recovery proof, and it surfaced one real restart bug that had to be fixed in
the host implementation instead of hidden behind weaker assertions.

The production change is twofold:

- local and server demo boot assemblies can now opt into a test-only
  broker-backed real plugin format through explicit env overrides for plugin
  format, plugin type id, and scan root
- both hosts now persist active sandbox specs and actually re-establish
  broker-backed AU, VST3, and LV2 sessions during `restart_plugin_sandbox(...)`
  instead of only recording a `SandboxRestarted` lifecycle marker

That enabled focused public broker-backed recovery proof:

- `signal-host-local/tests/public_host_edge_sandbox_broker.rs` now proves a
  broker-backed VST3 crash-recovery boot path
- `signal-host-server/tests/public_host_edge_sandbox_broker.rs` now proves a
  broker-backed LV2 crash-recovery boot path

The local broker process helper was also hardened so direct broker-process
tests clear and restore the demo-plugin override env while they run. That keeps
the new recovery-proof env overrides from contaminating the direct
`run-demo`/`run-timeout-demo` broker checks in the same test binary.

This is a meaningful step because Batch 2.3 now has repo-owned proof for
shared-process recovery behavior, not only steady-state broker ensure and
teardown lanes. It also flushes out the fact that sandbox restart had not been
re-establishing broker-backed sessions, which is now corrected for the formats
already on the broker path.

Focused validation passed for:

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Batch 2.3 Tranche 17 Outcome

This tranche broadened Batch 2.3 proof from one successful broker-backed
restart lane per host into the first broker-backed recovery failure proof.

The implementation change stayed intentionally small and proof-oriented:

- `signal-host-local/tests/public_host_edge_sandbox_broker.rs` now proves a
  broker-backed VST3 overlap-contention recovery abort through the public boot
  surface
- `signal-host-server/tests/public_host_edge_sandbox_broker.rs` now proves the
  same recovery-abort shape for broker-backed LV2 on the server host

Those proofs verify the shared broker process contract on the failure side, not
just on the happy restart side:

- the boot path still attaches a broker-backed sandbox and records the same
  broker lease attachment evidence before recovery begins
- overlap contention aborts with the expected
  `RuntimeErrorKind::ResourceUnavailable`
- both hosts stop with `DegradedModeRecovery`, clear active sessions back to
  zero, and record the expected overlap-session rejection reason through the
  public supervisor report

This matters because Batch 2.3 is no longer proving only steady-state broker
ensure/teardown plus one successful crash restart. It now also proves that the
shared broker-backed recovery path surfaces contention failure truth cleanly
through the public host edge on both hosts.

Focused validation passed for:

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Batch 2.3 Tranche 18 Outcome

This tranche pushed the public proof surface one level deeper into teardown-
stage ownership truth instead of stopping at overlap admission failure.

The new public broker-backed proofs now cover deferred old-transport teardown
failure on both hosts:

- `signal-host-local/tests/public_host_edge_sandbox_broker.rs` now proves a
  broker-backed VST3 deferred-teardown recovery abort
- `signal-host-server/tests/public_host_edge_sandbox_broker.rs` now proves the
  same deferred-teardown abort shape for broker-backed LV2

Those proofs verify the public recovery state after broker-backed teardown
failure, not just the top-level runtime error:

- the broker-backed boot path still reaches the shared lease-attached state
  before recovery begins
- deferred old-transport teardown aborts with the expected
  `RuntimeErrorKind::ResourceUnavailable`
- both hosts stop with `DegradedModeRecovery` while still exposing one active
  sandbox, one attached session, one lingering session, and one detach-faulted
  session in the exported transport concurrency truth
- the exported public report proves that the failing retained transport is
  surfaced as `DetachFaulted` rather than silently disappearing

This materially tightens the remaining Batch 2.3 scope. The queue now has
repo-owned public proof for:

- steady-state broker attach and teardown
- successful broker-backed crash restart
- overlap-contention recovery failure
- deferred old-transport teardown failure

Focused validation passed for:

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Batch 2.3 Tranche 19 Outcome

This tranche completed the public proof envelope for Batch 2.3 and made the
closeout threshold explicit instead of leaving the queue open-ended.

The new public broker-backed proofs now cover the cleanup-retry success path on
both hosts:

- `signal-host-local/tests/public_host_edge_sandbox_broker.rs` now proves a
  broker-backed VST3 deferred-cleanup-retry recovery success
- `signal-host-server/tests/public_host_edge_sandbox_broker.rs` now proves the
  same deferred-cleanup-retry recovery success for broker-backed LV2

Those proofs verify the success-side ownership truth after a teardown-stage
recovery problem has already occurred:

- both hosts restart back to `processing_epoch == 2` with `restart_count == 1`
- the public control snapshot returns to `running == true`
- lingering and detach-faulted session counts fall back to zero
- one attached active session remains exported as `AttachActive`
- the public broker failure history still preserves the injected lingering
  cleanup retry fault instead of erasing it from the recovered report

That gives Batch 2.3 a complete enough public proof envelope to stop:

- steady-state broker attach and teardown
- successful broker-backed crash restart
- overlap-contention recovery failure
- deferred old-transport teardown failure
- deferred cleanup-retry recovery success

With that envelope in place, Batch 2.3 is now complete and the remaining work
belongs to Batch 2.4: making deferred gaps explicit instead of continuing to
grow the proof matrix inside the ownership-hardening queue.

Focused validation passed for:

- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Batch 2.4 Tranche 1 Outcome

This tranche finished the remaining explicit-gap work and closes `g09.002`
instead of leaving the cross-host proof queue open on one vague final checkbox.

The host-facing deferred gaps now fail explicitly instead of succeeding
silently:

- `signal-host-local` now turns unsupported or undiscovered sandbox ensure
  requests into explicit `InvalidRequest` errors while recording the attempted
  sandbox as a protocol-violation fault in runtime-owned lifecycle state
- `signal-host-server` now does the same for its own unsupported or undiscovered
  sandbox ensure requests

That behavior is now proven through one public host-edge deferred-gap lane per
host:

- `signal-host-local/tests/public_host_edge_cross_adapter_parity.rs` now proves
  that CLAP scan truth is still exported while CLAP sandbox ensure fails
  explicitly with a recorded runtime fault on the local host
- `signal-host-server/tests/public_host_edge_cross_adapter_parity.rs` now
  proves the same explicit CLAP sandbox gap on the server host

This is enough to close Batch 2.4 for `g09.002` because the remaining bounded
plugin-hosting gaps are no longer implicit behavior:

- real scan/load/process smoke proof exists for the supported host/format lanes
- both hosts consume the same runtime-owned broker substrate
- at least one real deferred gap per host is now surfaced explicitly through
  public scan/load receipts instead of via silent no-op ensure behavior

Focused validation passed for:

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_cross_adapter_parity`
- `cargo test -p signal-host-server --test public_host_edge_cross_adapter_parity`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

COMPLETED: `g09.002` is now closed.

Next Task

Start `g09.003` by replacing the remaining VST3 scaffold depth behind the now-
finished shared substrate: inventory the still-scaffolded VST3 discovery,
component-loading, and lifecycle boundaries, then land the first tranche that
replaces scaffold descriptor hydration with real module introspection while
keeping the existing broker-backed host proofs green.
