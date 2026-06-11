# Post-g10 Rebuild-On-Demand Queue

Status: backlog
Created: 2026-06-11

Items deliberately NOT scheduled in g10. Each is pulled into a future
generation only when a Loophole product feature needs it — never
speculatively. The g10 demolition exists precisely so these rebuilds start
from honest foundations.

## Real plugin hosting (CLAP-first)

Foundation kept by g10.007: CLAP discovery FFI, VST3 introspection, contracts
vocabulary, block transport + watchdog, shared-memory broker plumbing.
Missing (in order): instance lifecycle FFI (`factory->create_plugin`,
`init/activate/start_processing`); RT `process()` bridging over shared memory
with an audio-thread loop in the sandbox process (semaphore/futex signaling,
not line-oriented stdin); real event translation
(`clap_input_events`/`clap_output_events`); state-chunk round-trips; crash
isolation wired to real child exit status; GUI embedding (`clap.gui` →
window-handle plumbing through the host). VST3 via the `vst3` crate rather
than extending hand-rolled COM; AU/LV2 last, behind the same contracts.

## Engine server / out-of-process engine

Nothing kept from signal-host-server (it was a print-and-exit clone). If
wanted: design transport + session dispatch fresh on signal-ipc shared memory.

## Device handling depth

Device-change notifications, input/duplex streams for recording, explicit
device selection UI contract. Builds on g10.003's cpal enumeration.

## Resampling/time domain

Disk streaming for long media; time-stretch; higher-quality SRC tiers beyond
the g10.008 polyphase table.

## Beat tracking upgrade

Replace fixed-grid beat placement with DP/HMM tracking for drifting tempo;
widen the 70-180 BPM default range. Builds on the rhythm core kept by
g10.006.

## Graph successor

Production node-graph execution designed around the render plane's
control/render split (topological ordering, PDC via delay insertion, retained
stage state, preallocated buffers). Nothing from signal-graph's execution
path is reusable; its telemetry vocabulary may inform the control side.

## Multichannel/loudness breadth

Surround channel weights (1.41 Ls/Rs) and true multichannel metering when
Loophole grows beyond stereo.
