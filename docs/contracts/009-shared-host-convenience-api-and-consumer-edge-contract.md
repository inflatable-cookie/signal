# 009 Shared Host Convenience API And Consumer-Edge Contract

Status: active
Owner: core-product
Updated: 2026-03-12
Related contracts: `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`, `docs/contracts/006-runtime-hardware-portability-and-clock-domain-contract.md`, `docs/contracts/008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared host-edge stability boundary for `g05.002` so Signal
can expose useful host convenience APIs without letting host-local helpers
become a second authority alongside the runtime/export/plugin contracts already
frozen in `g04` and widened in `g05.001`.

## Authority hierarchy

Shared host convenience APIs have one authority chain:

1. `signal-runtime` owns lifecycle, projection, observation, and supervisor
   meaning through:
   - `RuntimeLifecycleApi`
   - `RuntimeProjectionApi`
   - `RuntimeObservationApi`
   - `RuntimeSupervisorApi`
   - runtime-owned observation, supervisor, host-I/O, profiling, and soak
     report families
2. host crates such as `signal-host-local` and `signal-host-server` may expose
   convenience entry points only when those entry points stay thin wrappers or
   typed enrichments over runtime-owned meaning
3. backend crates own negotiation, callback, transport, and protocol detail,
   but they must not define the shared consumer contract for host lifecycle,
   reports, or supervisor orchestration

If a consumer-visible host edge cannot be explained in terms of runtime-owned
traits or typed runtime-owned report surfaces, it is not yet a shared stable
host contract.

## Stability tiers

This milestone uses the maturity language frozen in contract `003`, but applies
it to host-edge APIs instead of whole crates.

### `public` shared host edge

The following host surfaces are part of the first shared host-edge contract:

- `LocalRuntimeHost::new` and `ServerRuntimeHost::new`
- `RuntimeSupervisorApi` as implemented by both host types:
  - plugin scan and sandbox orchestration
  - recording/media/warp/clip reconciliation
  - offline render, queue, checkpoint, purge, and delegated-execution
    preparation/finalization entry points
  - backend-policy and sandbox restart/teardown requests
- `LocalRuntimeHost::supervisor_report` and `ServerRuntimeHost::supervisor_report`
  returning `RuntimeSupervisorReport`

These edges are `public` because they satisfy all of the following:

- they exist across the current shared host implementations rather than on one
  backend path only
- they delegate to or return runtime-owned contracts instead of inventing
  host-local DTOs as the source of truth
- they are narrow enough for later conformance and packaging work to exercise
  without depending on scenario-specific host fixtures

### `consumer-facing but unstable`

The following host surfaces may be useful to early consumers, but they are not
yet part of the shared stable host-edge promise:

- `LocalRuntimeHost::observation_report`
- `LocalRuntimeHost::host_observation_report`
- `LocalRuntimeHost::host_supervisor_report`
- `ServerRuntimeHost::observation_report`
- `LocalRuntimeHost::runtime`
- `LocalRuntimeHost::clap_supported`
- `LocalRuntimeHost::finalize_offline_render_with_local_delegated_executor`
- `LocalRuntimeHost::render_offline_with_local_delegated_executor`
- `LocalRuntimeHostSummary`
- `ServerRuntimeHostSummary`

These surfaces remain unstable for one or more of these reasons:

- they are asymmetric across current host implementations
- they mix runtime-owned meaning with backend- or host-specific enrichment that
  still lacks a shared parity contract
- they expose convenience around a local delegated executor or demo assembly
  rather than a backend-neutral host promise

Consumers may experiment with these APIs, but packaging, conformance, and later
release claims must not treat them as frozen shared edges yet.

### `internal` or scenario-only helpers

The following families remain outside the shared consumer contract:

- `boot_*` scenario helpers on local and server hosts
- fault-injection, watchdog-soak, recovery-drill, and demo-assembly helpers
- private broker, transport, callback, and runtime-assembly helpers that exist
  only to support host fixtures or backend realization detail

These helpers may remain public for tests or examples, but they should be
treated as scenario fixtures rather than reusable stable consumer APIs.

## Shared host-edge promises

The first shared host-edge boundary keeps four promises.

### Stable host edges stay thin

A stable host convenience API must remain a thin wrapper over runtime-owned
meaning. It may:

- delegate directly into runtime-owned lifecycle, projection, observation, or
  supervisor traits
- return runtime-owned reports or receipts
- add host-specific context only through typed runtime-owned host report DTOs

It must not:

- become the primary source of truth for lifecycle or capability state
- require consumers to reconstruct runtime meaning from host-private summary
  structs
- hide essential runtime/export contracts behind backend-specific convenience

### Symmetry matters for stability

A host edge should not be promoted to shared-stable status unless it is either:

- available across the shared host implementations, or
- promoted as an explicitly asymmetric contract with a documented rationale and
  typed runtime-owned surface

This rule is why `supervisor_report()` is stable today while
`host_supervisor_report()` and host-specific summary structs are not.

### Host-specific enrichment must stay additive

When a host enriches runtime state with hardware, clock, or callback context,
that enrichment must remain additive to runtime-owned reports rather than a
replacement for them. `RuntimeHostObservationReport` and
`RuntimeHostSupervisorReport` are the correct shape for that enrichment; host
summary structs are explanatory only until a broader parity contract exists.

### Scenario helpers do not imply API stability

Boot helpers, recovery drills, soak entry points, and delegated-executor
fixtures may remain useful in tests, examples, or supervisor tools, but their
public presence does not promote them into the shared consumer boundary by
default. Stability requires an explicit contract update.

## Canonical consumer order

Consumers should inspect and use host edges in this order:

- use `signal-runtime` directly when only runtime-owned lifecycle, projection,
  observation, or supervisor meaning is required
- use shared host-edge APIs only when host orchestration is genuinely needed
  and the API delegates back to runtime-owned meaning
- use host-specific enriched reports only when backend or hardware context is
  required, and treat those surfaces as unstable unless the contract promotes
  them explicitly

The host layer is a convenience boundary, not a replacement authority.

## Deferred host-edge breadth

This Batch 2.1 contract intentionally defers:

- promotion of `RuntimeHostObservationReport` / `RuntimeHostSupervisorReport`
  into the shared stable tier across all hosts
- any promise around `LocalRuntimeHostSummary` / `ServerRuntimeHostSummary` as
  reusable consumer DTOs
- scenario boot helpers as product-facing orchestration APIs
- local delegated executor convenience methods as backend-neutral host edges
- stronger host-edge conformance fixtures until receipt/export alignment work
  lands in Batch 2.2 and Batch 2.3

Those areas remain useful, but they are not yet part of the frozen shared host
boundary.

## Current inspection surface

Batch 2.2 now exposes the shared host-edge boundary through a repo-owned,
machine-readable inspection surface rather than prose alone:

- `cargo run -p signal-supervisor-tools -- --describe-host-edge-boundary --format=json`
- `effigy acceptance:host-edge-consumer`

## Current consumer proof surface

Batch 2.3 now proves the stable shared host edge through public consumer paths
rather than private host internals:

- `crates/signal-host-local/tests/public_host_edge_boundary.rs`
- `crates/signal-host-server/tests/public_host_edge_boundary.rs`
- `effigy acceptance:host-edge-consumer`

## Next Task

Continue `g05.005` with Batch 5.1 by defining the combined `g05`
generation-closeout descriptor and task without promoting unstable host
helpers into the widened shared boundary.
