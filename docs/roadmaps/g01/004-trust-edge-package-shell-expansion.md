# Roadmap g01.004: Trust-Edge Package Shell Expansion

Status: active
Owner: core-product
Created: 2026-03-08
Depends on: g01.003
Vision tags: RT, RES, AUTH
Target envelope: expand the initial Rust workspace shell with the first
trust-edge package set so runtime-host, plugin-sandbox, hardware, and shared
control/message boundaries exist as real workspace packages rather than docs
only.

## Problem

Signal's core workspace shells existed, but the trust-edge boundaries still
lived only in architecture docs. That left the runtime-host split incomplete in
implementation terms:

1. plugin and hardware boundaries had stable names but no real package homes,
2. host binaries could not point at trust-edge crates directly,
3. runtime-host and sandbox ownership rules were still easier to drift on than
   the shared DSP and analysis crates.

## Goals

- add the first trust-edge package shells:
  - `signal-ipc`
  - `signal-plugin`
  - `signal-plugin-clap`
  - `signal-plugin-sandbox`
  - `signal-hardware`
  - `signal-hardware-coreaudio`
- wire those packages into the root Cargo workspace
- make host ownership boundaries visible in `signal-host-local` and
  `signal-host-server`
- update Signal-owned docs so the package map and active roadmap reflect the
  real workspace state

## Non-Goals

- implementing real CLAP hosting or CoreAudio I/O in this batch
- defining the final shared-memory sandbox layout in code
- replacing the legacy C++ tree in this batch

## Execution Plan

### 004.1 Workspace expansion

- [x] Add the trust-edge packages to the Cargo workspace.
- [x] Create minimal manifests and source files for each package.

### 004.2 Host-boundary wiring

- [x] Point `signal-host-local` at the plugin and hardware shells.
- [x] Point `signal-host-server` at the plugin and hardware shells.
- [x] Keep the code intentionally small but explicit about ownership boundaries.

### 004.3 Documentation alignment

- [x] Update the package map to reflect the current workspace state.
- [x] Add this milestone to the active roadmap index.
- [x] Update the repo/docs entry points to point at the trust-edge expansion.

## Acceptance Signals

1. `cargo check --workspace` succeeds with the new trust-edge packages present.
2. A contributor can see where plugin, sandbox, hardware, and shared
   control/message code belong without inferring it from Loophole docs.
3. The host binaries reference the trust-edge packages clearly enough that the
   runtime-host ownership split is visible in code.

## Next Task

Push the executable engine slice beyond requested/effective forecast mode
reconciliation by teaching `signal-runtime` to propagate those transitions into
the real scheduler path, especially queued prework retirement or reuse when
mode changes, recovery crosses an anticipative-disabled epoch, or runtime role
defaults are revised while future work is already primed. The next concrete
step after that is to let runtime rebuild the future window proactively after
forecast-plan changes instead of only preserving compatible queued entries and
retiring the rest.
now explicit runtime behavior instead of silent overwrite.
The next practical step is no longer "make queued prework exist" but "make the
bounded queue behave more like a scheduler": hosts now keep a small future
priming horizon, and runtime already distinguishes identical future-state reuse
from changed future-state replacement for the same target block.
That has now moved one step further: the host horizon is expressed to runtime
as an explicit planning window, and runtime can retire queued future targets
that fall out of that window with `PlanningWindowRevised` instead of relying
on host-local queue replacement alone.
That queue reconciliation is now proactive rather than passive: when forecast
mode/profile/policy changes revise the future plan, runtime preserves
compatible queued work, retires incompatible entries, and re-primes any
missing future targets immediately.
That lifecycle ownership has now widened too: runtime can rebuild the current
forecast window on graph apply and start/restart, so host boot no longer needs
its own one-off forecast priming call to seed anticipative work.
The next scheduler seam after that was whether the future window represented
immediately prepared work or a real bounded background-lane queue. That is now
moving into runtime too: future targets can remain pending inside the runtime
window until the bounded prework runner prepares them.
That runner is now also separated from current-block forecast application:
forecast application reconciles the future window, while an explicit
runtime-owned prework service lane advances pending future targets.
That lane now also exposes runtime-owned paused, pending, servicing, idle,
disabled, and starved states, so the next scheduler work can build on a
stateful background service boundary rather than a blind repeated drain call.
That stateful lane now also has explicit pressure hints and adaptive service
behavior, so timeout/watchdog paths can make runtime throttle or yield
anticipative work instead of servicing it with a fixed drain policy.
It now also has backlog-aware prioritization, so elevated pressure keeps
servicing immediate and near-term pending targets while deferred future work
stays queued instead of being drained in plain block order.
That elevated-pressure policy is now also graph-semantic-aware, so latency-
focused anticipative graphs can widen that service scope without changing the
host block loop or introducing host-local scheduler branching. Mixed graphs
with plugin-backed realtime nodes now also narrow that service scope into a
plugin-constrained path without pushing scheduler exceptions back into hosts.
That plugin-constrained path is now also sandbox-aware, so active plugin
sandbox count can fully gate the background prework lane under non-normal
pressure and the shared engine snapshot/export surface now reports both the
active plugin sandbox count and whether that gate is active.
That boundary now also carries plugin-backed node bindings, so runtime can
connect plugin-constrained scheduling to the live transport-session state of
the specific sandbox a plugin-backed realtime node belongs to instead of
making that decision from graph shape and a global sandbox count alone.
That host/runtime seam is now exercised through host graph assemblies that
declare graph projection, plugin sandbox inventory, and plugin-backed node
bindings together, so the runtime scheduler is no longer fed by a separate
demo-only binding helper path.
The next host-owned seam after that was the future-state forecast itself.
That is now moving into `signal-runtime` too: hosts provide a compact forecast
policy, and runtime derives the future transport projection, parameter batch,
and primed input block for planned window targets instead of hosts
hand-building every future target object.
That same policy now also owns the planning-window size, so hosts no longer
carry a separate horizon constant or remaining-block cap, they no longer run a
separate forecast-advance step, and they no longer resend the same policy on
each block before asking runtime to manage future work.
