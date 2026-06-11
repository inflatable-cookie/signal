# 003 - Output Stream Hardening And Real Device Enumeration

Status: active
Owner: core-product
Created: 2026-06-11
Depends on: g10.002
Vision tags: `HARDWARE`, `OUTPUT-PATH`, `PORTABILITY`

## Problem

The cpal output backend force-overrides the device's default config with the
requested rate/channels and reports the requested values back — a 44.1
kHz-only or mono device fails to open, and a silent mismatch is undetectable
downstream. Its `unsafe impl Send` for the stream wrapper is
platform-unconditional while the safety argument is macOS-specific; cpal's
`Stream` is `!Send` for real reasons on other backends. The crate has zero
tests.

Separately, `signal-hardware-coreaudio` is a `system_profiler` JSON parser:
a 1-3 s subprocess at construction, name-slug device IDs that reshuffle on
reorder, invented buffer-size lists, and `simulate_*` mutators on the public
API. cpal — already a dependency — can enumerate real devices with real
configs.

Host-side consequences (Loophole's Aura host never re-checks stream
rate/channels against a new plan, ignores seek-while-playing, never reopens a
faulted stream) are coordinated from the Loophole side (chorus g11+), but the
contract changes that make those fixes possible land here.

## Goals

- [x] config negotiation in `CpalOutputBackend`: try requested spec, fall back
      to nearest supported (rate, then channels), report the *negotiated*
      config on the handle
- [x] `OutputStreamHandle` exposes negotiated sample rate and channel count so
      hosts can detect plan/stream mismatch
- [x] gate the `unsafe impl Send` to `target_os = "macos"` with a documented
      safety argument; provide a portable alternative (stream owned by a
      dedicated thread with a command channel) or a compile error on other
      platforms until one exists
- [x] device enumeration via cpal (names, supported configs, default device)
      replacing the system_profiler path
- [ ] retire `signal-hardware-coreaudio` once no consumer needs it
- [x] smoke-test coverage for the backend (skippable when no device present)

## Non-Goals

- [ ] no input/duplex streams yet (recording pulls that later)
- [ ] no device-change notifications (backlog with the rebuild items)

## Execution Plan

### Batch 3.1 - Negotiation And Honest Reporting

- [ ] supported-config matching with deterministic fallback order
- [ ] negotiated config on the handle; error type for "no usable config"
- [ ] CI-skippable open/close smoke test

### Batch 3.2 - Send Soundness

- [ ] macOS-gated `unsafe impl` with safety comment, portable path or
      explicit unsupported-platform error

### Batch 3.3 - Enumeration Cutover

- [ ] cpal-backed device inventory in `signal-hardware` (or the cpal crate)
- [ ] migrate any consumers off `signal-hardware-coreaudio`; delete the crate
- [ ] workspace build + full test gate

## Acceptance Criteria

- [ ] requesting 48k/stereo on a default device opens and reports the actual
      negotiated values
- [ ] no `system_profiler` subprocess anywhere in the workspace
- [ ] Loophole's Aura host can read negotiated config (verified by a
      host-side follow-up in chorus g11)

## Risks and Mitigations

- Risk: negotiated-rate playback needs host cooperation (plan compiled at
  another rate).
- Mitigation: handle exposes the truth; render-plane install validates
  (g10.002 batch 2.3); host recompiles plans at the negotiated rate.

## Evidence Requirements

- [ ] smoke test output on real hardware recorded in the progress log

## Progress (2026-06-11)

- Batches 3.1 + 3.2 + most of 3.3 landed: `negotiate_config` matches the
  requested spec against the device's supported output configs (exact
  channels + exact rate, then nearest rate, then device default) and the
  handle reports the negotiated values, never echoes. The `unsafe impl Send`
  is GONE entirely — each stream is owned by a dedicated thread that builds,
  starts, parks, and drops it; the handle talks to it over channels, so the
  crate is portable with zero unsafe. `enumerate_output_devices()` lists
  real devices with default/supported rates and channel counts from cpal.
  Two CI-skippable smoke tests (open+negotiate, enumerate) pass on real
  hardware. Output-stream contract doc updated: negotiation replaces
  honour-or-fail; callers must read negotiated values from the handle.
- Host side (Loophole Aura, chorus g11 runway): plans now compile at the
  stream's negotiated rate (`render_plan_spec(cache, sample_rate_hz)` in
  pulse, default 48k before a stream exists; sync recompiles once when the
  negotiated rate differs); `ensure_stream` reopens on fault or channel
  mismatch and feeds `set_stream_channels`; seek-while-playing now reaches
  the render plane (>100 ms authority/clock divergence treated as an
  explicit declicked seek); transport command errors recorded instead of
  discarded.
- Remaining in this packet: retire `signal-hardware-coreaudio`. Deferred —
  its only consumer is signal-host-local's boot path, which g10.005
  collapses; deleting it rides that packet.

## Next Task

g10.004 (hosting demolition) — with the real path hardened, remove the fake
one. g10.005 finishes this packet's coreaudio retirement.
