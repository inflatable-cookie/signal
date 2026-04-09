# 004 - Real AU Discovery, CoreAudio-Backed Execution, And macOS Proof

Status: complete
Owner: core-product
Created: 2026-04-08
Depends on: g09.002
Vision tags: `PLUGIN`, `AU`, `COREAUDIO`
Contract refs: `021`, `073`

## Problem

The AU adapter and the CoreAudio backend are both still scaffolded. Signal
therefore cannot honestly claim a real macOS plugin and device path even
though the contracts and crate layout suggest it can.

## Goals

- [x] implement real AU discovery and lifecycle over AudioComponent traversal
- [x] replace CoreAudio fake-device behavior with real enumeration and
      diagnostics
- [x] prove one runtime-owned macOS path from plugin scan to device-backed
      execution

## Non-Goals

- [ ] no AU editor-view hosting
- [ ] no product-local macOS device picker UX

## Execution Plan

### Batch 4.1 - CoreAudio Device Truth

- [x] implement bounded real CoreAudio device enumeration, identity, endpoint,
      and diagnostics projection
- [x] thread the new CoreAudio device truth through stable host-edge receipts
- [x] demote the synthetic default-device path to test-only behavior
- [x] expose unavailable, degraded, duplex-mismatched, and healthy states
      through shared hardware/runtime receipts

### Batch 4.2 - AU Discovery And Bring-Up

- [x] implement bundle-local AU metadata scan roots and component-description
      projection
- [x] instantiate AU units through the hardened sandbox process
- [x] map initialization, bus-layout, and render-context failure into runtime
      lifecycle and fault receipts

### Batch 4.3 - macOS End-To-End Proof

- [x] prove one AU plus CoreAudio execution path through runtime and stable
      host-edge receipts
- [x] add focused macOS smoke tasks or acceptance descriptors for this path
- [x] defer one interactive AU/CoreAudio demo scenario into the dedicated demo
      substrate milestones `g09.011` and `g09.012`

## Acceptance Criteria

- [x] CoreAudio no longer answers with synthetic default-device truth
- [x] AU discovery and execution are real and runtime-owned
- [x] Signal has one honest macOS plugin-plus-device proof path

## Risks And Mitigations

- Risk: CoreAudio realization bleeds backend-private detail into shared DTOs.
- Mitigation: map only device, endpoint, and diagnostics meaning into the
  shared contract.

- Risk: AU and CoreAudio work drift apart into separate partial proofs.
- Mitigation: keep the milestone focused on one end-to-end macOS path.

## Evidence Requirements

- [x] log CoreAudio and AU tranches separately
- [x] run `cargo check -p signal-plugin-au`
- [x] run `cargo check -p signal-hardware-coreaudio`
- [x] run `effigy health`

## Batch 4.1 Tranche 1 Outcome

The CoreAudio backend no longer answers with a hard-coded fake default device
inside `signal-hardware-coreaudio`. The backend now reads real bounded device
inventory from `system_profiler SPAudioDataType -json`, normalizes discovered
device identity into shared `AudioDeviceDescriptor` records, and sets baseline
healthy versus degraded backend diagnostics from the presence or absence of a
default output device. The old synthetic default-device identity is gone from
production enumeration, while a fixture-backed inventory override keeps the
backend crate tests deterministic.

This tranche deliberately stopped at the backend and host contract boundary.
`signal-host-local` compiles against the new CoreAudio truth, but the stable
local public host-edge proof lane is currently blocked by a pre-existing
`boot_default()` failure in the CLAP sandbox path (`plugin format Clap is not
supported here yet on the local host sandbox path`). That failure is outside
the CoreAudio inventory realization itself, so the next meaningful batch is to
move the macOS hardware proof off that blocked default boot path or to repair
the default local host boot path before widening AU execution depth.

## Batch 4.1 Tranche 2 Outcome

The macOS hardware proof lane is now stable again without reopening the
explicit local CLAP sandbox gap. The public local host-edge hardware proofs now
boot through the supported AU demo-plugin override path, which means the new
CoreAudio device truth is exercised through a runtime-owned host surface rather
than remaining backend-only. This closes the immediate proof gap from the first
CoreAudio tranche while keeping the local host explicit that CLAP sandbox
ownership is still deferred on that path.

What remains open in `g09.004` is no longer CoreAudio device identity or basic
host-facing hardware proof. The next major seam is deeper AU execution truth:
instantiating AU units through the hardened sandbox path and proving one honest
AU-plus-CoreAudio runtime-owned execution lane.

## Batch 4.2 Tranche 1 Outcome

The first real macOS plugin seam is now in place. `signal-plugin-au`
production discovery reads bundle-local `signal-au-component.txt` metadata
instead of inferring AU identity from bundle names or scaffold lookups, and the
host-local and host-server AU proof roots now materialize the same metadata
contract in temp `.component` bundles. Internal `signal-host-server` AU scan
helpers were brought onto the same path so the baseline AU proof surface is
consistent across adapter tests, host-internal scan tests, and public host-edge
proofs.

This tranche deliberately stops at discovery and bounded bring-up. The AU lane
is now honest about plugin identity and descriptor shape, but it still does not
instantiate through a real AU execution core or exercise a real CoreAudio
device path. The next meaningful seam is CoreAudio device truth or AU unit
instantiation depth, not more scan-root churn.

## Batch 4.2 Tranche 2 Outcome

The AU lane now crosses a real runtime-owned sandbox bring-up path instead of
falling back to the generic demo broker path. `signal-plugin-au` now exposes
bounded state-store, activation, and teardown records derived from discovered
AU metadata, `signal-plugin-sandbox` now has an explicit AU broker flavor with
`attach-au`, `run-au`, and `teardown-au`, and both `signal-host-local` and
`signal-host-server` feed the AU broker lane with real bundle-root and
plugin-type identity plus lifecycle summaries. The stable public AU host-edge
proofs now run through that brokered AU path and require AU lifecycle detail in
the exported supervisor report instead of only asserting that some sandbox was
attached.

This tranche intentionally stops at bounded AU lifecycle truth rather than full
AU render execution parity. The remaining large seam in `g09.004` is explicit
AU failure mapping and one tighter AU-plus-CoreAudio proof that carries device
truth and AU lifecycle truth through the same runtime-owned path.

## Batch 4.2 Tranche 3 Outcome

The AU lane now fails explicitly at the right bounded bring-up seams instead of
collapsing back into generic invalid-request behavior. `signal-plugin-au`
metadata can now declare initialization, bus-layout, and render-context fault
contracts, the AU adapter maps those to `instantiate_plugin(...)`,
`prepare_session(...)`, and `activate_instance(...)` failures, and both host AU
sandbox paths record the resulting lifecycle and fault truth into runtime-owned
sandbox state before returning the error. On the proof side, the stable local
AU host-edge surface now includes a fault lane that boots through the real
CoreAudio device path while a faulty AU bundle fails during render-context
activation, proving that the exported host report carries both real device
truth and AU fault truth from the same runtime-owned surface.

This closes the main AU fault-explicitness seam in `g09.004`. What remains is
not more adapter lifecycle plumbing; it is milestone promotion work around a
tighter macOS smoke/acceptance descriptor and the later demo substrate lane.

## Batch 4.3 Tranche 1 Outcome

`g09.004` now has a repo-owned macOS acceptance surface instead of a loose set
of proof commands. `effigy acceptance:macos-au-coreaudio-boundary` ties
CoreAudio backend enumeration, runtime-owned AU lifecycle truth, runtime-owned
external-I/O truth, and the stable local host-edge AU/CoreAudio proofs into one
stable acceptance task. `signal-supervisor-tools` now exposes the same lane
through `--describe-macos-au-coreaudio-boundary --format=json`, so downstream
consumers can inspect the focused macOS plugin-plus-device boundary without
reading host, adapter, or backend internals.

This is also the right promotion point. The remaining bounded AU omissions are
now deliberate scope, not hidden scaffold: editor hosting, deeper parameter
tree breadth, and richer demo or operator workflows already belong to the later
demo milestones rather than this realization milestone. `g09.004` therefore
closes as the first honest AU-plus-CoreAudio implementation and acceptance
boundary, while the interactive operator story is explicitly handed off to
`g09.011` and `g09.012`.

## Next Task

Start `g09.005` with one meaningful Linux plugin-realization batch: audit the
remaining LV2 scaffold seams in discovery, extension negotiation, and host
proof roots, then land the first production-depth pass on real LV2 bundle and
extension identity before widening worker or live execution behavior.
