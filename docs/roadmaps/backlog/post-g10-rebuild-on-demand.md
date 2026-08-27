# Post-g10 Rebuild-On-Demand Queue

Status: backlog
Created: 2026-06-11
Updated: 2026-08-17

Items deliberately NOT scheduled in `g10`. Each is pulled into a future
generation only when a Loophole product feature needs it — never
speculatively. The `g10` demolition exists precisely so these rebuilds start
from honest foundations.

## Plugin hosting — baseline shipped

**Do not treat this section as a rebuild queue.** Real plugin hosting for
CLAP, VST3, AU, and LV2 is implemented and tested. The June 2026 wording
below was pre-`g09`/`g11` and is kept only as historical context.

### Shipped today

- `signal-plugin-clap`, `signal-plugin-vst3`, `signal-plugin-au`, and
  `signal-plugin-lv2` — real discovery, lifecycle, `process()`, events, state,
  and GUI paths (format-specific detail stays adapter-local until promoted)
- `signal-plugin-sandbox` — long-lived broker child process with shared-memory
  block transport, crash isolation, and format coverage tests including real
  system AU components on macOS
- `signal-plugin-bridge` (`g11.012+`) — placement-agnostic render-plane
  backends:
  - **InProcess** tier: direct FFI/process on the audio thread (CLAP, VST3,
    AU, LV2)
  - **DedicatedSandbox** tier: one plugin per child process,
    `ShmPluginProcessor`, bounded wait, bypass-on-miss
- offline render-plane integration — plugin stages can drive real backends in
  tests and offline render proofs

Authoritative implementation reference: `crates/signal-plugin-bridge/src/lib.rs`
and `crates/signal-plugin-sandbox/tests/plugin_hosting/`.

### Still deferred (product-pull only)

- **Product browser / workflow shells** — inventory UX and downstream-app
  workflow remain outside Signal unless explicitly promoted

Pulled into `g11` (do not treat as backlog):

- **Production host-assembly wiring** — closed in `g11.001`
- **SharedSandbox tier** — closed in `g11.002`; map at
  `docs/architecture/shared-sandbox-multiplexing.md`

Historical note (2026-06-11, superseded): the original post-demolition list
named missing lifecycle, RT `process()` bridging, events, state, crash
isolation, and GUI as future work. That work landed through `g09` and later
hosting batches. Do not reopen it from this backlog item.

## Engine server / out-of-process engine

Nothing kept from `signal-host-server` (it was a print-and-exit clone). If
wanted: design transport + session dispatch fresh on `signal-ipc` shared memory.

## Device handling depth

Device-change notifications, input/duplex streams for recording, explicit
device selection UI contract. Builds on `g10.003`'s cpal enumeration.

## Resampling/time domain

Higher-quality SRC tiers beyond the `g10.008` polyphase table remain deferred.

## Beat tracking upgrade

Replace fixed-grid beat placement with DP/HMM tracking for drifting tempo;
widen the 70-180 BPM default range. Builds on the rhythm core kept by
`g10.006`.

## Graph successor

Production node-graph execution designed around the render plane's
control/render split (topological ordering, PDC via delay insertion, retained
stage state, preallocated buffers). Nothing from `signal-graph`'s execution
path is reusable; its telemetry vocabulary may inform the control side.

## Multichannel/loudness breadth

Surround channel weights (1.41 Ls/Rs) and true multichannel metering when
Loophole grows beyond stereo.

## Next Task

Promote one deferred item only when the operator selects it in
`docs/roadmaps/strategic-runway.md` or an active generation milestone
requires it. Do not schedule plugin-hosting baseline work from this file.
