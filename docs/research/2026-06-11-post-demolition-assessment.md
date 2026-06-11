# Post-Demolition Assessment — 2026-06-11

Second-pass assessment of Signal after g10.002–009 (~98k LoC of simulated/
narration code removed, production path hardened). Two lenses: internal
architecture of the surviving engine seed, and the product-capability gap map
against what Loophole needs. Drives the g10 continuation packets (010+).

## Verdict

The seed is the right shape and the right size. signal-render-plane's
control/render split (immutable compiled plans, bounded mailboxes,
control-side dealloc, proven zero-alloc callback) is a sound foundation —
but it is a flat stereo lane/clip player, not an engine. The two structural
moves everything else stacks on: **graph-shaped plans** (node schedule with
plan-owned scratch buffers) and a **parameter fast path** (so continuous
gestures stop recompiling the world).

## Architecture findings (engine lens)

- **Plan-swap scaling cliff**: correct for topology edits, wrong for
  gestures. A fader drag at 60 Hz recompiles every clip and Kaiser table and
  ships whole plans through the mailbox. Fine at 10 tracks; breaks at
  hundreds of clips. Needs `SetParam` riding the existing mailbox against a
  compile-time parameter table.
- **State inheritance is O(lanes²) string compares on the audio thread**,
  and clips match by zip-index (breaks on mid-lane insert). Needs stable
  u64 node/clip identity + a control-side-precomputed inheritance map
  shipped inside the plan.
- **No node addresses** → no inserts, sends, PDC, metering targets, or
  automation targets. Graph-shaped plans give every later feature its
  addressing scheme. The retire/park lifecycle accommodates this unchanged.
- **Output time is dishonest**: cpal's `OutputCallbackInfo` is discarded;
  the playhead counts frames rendered, not played; the stream error callback
  drops the error detail. Recording alignment and drift handling need a
  frames→DAC-time mapping.
- **Zero engine observability**: SharedState exposes position/playing/
  parked only. No metering, no callback-health counters, no xrun inference.
- **DSP cupboard bare**: one-pole LPF only. No biquads, no pan law, no
  limiter, no dither, no denormal guard (low risk now, mandatory before
  feedback DSP).
- **Bounce ≠ playback**: signal-runtime's offline render runs the
  simulation graph with *linear* export resampling — a different mix than
  the render plane produces. Bounce must become "drive the same executor
  faster than realtime over the same plan".
- **signal-runtime endgame**: after bounce is re-founded, host-local (7.5k)
  + runtime (52.6k) can plausibly shrink under 10k — keep broker/sandbox
  sessions and the consumed observation slice. The prework scheduler is
  conceptually portable (anticipative rendering) but implementationally
  welded to the simulation; re-derive later as "pre-render plan regions
  ahead of the playhead", which `Samples` sources already know how to play.

## Product gap map (DAW lens)

| Capability | Today | v1 scope |
|---|---|---|
| Mixer realization | flat lanes×gain→master; **no pan at all**, Chain node graphs never reach audio | CORRECTED 2026-06-11: Loophole has no bus type — see the operator correction note below; chain-graph lowering is chorus g11.007 |
| Bounce/export | simulation-graph offline render (≠ playback) | S — run the real executor offline, WYSIWYG |
| Automation playback | pulse model real; engine has static gains | S–M — compiled breakpoint envelopes, gain first |
| Recording | nothing (cpal input never wired); pulse take/arm model waits | M + latency reporting prerequisite |
| Disk streaming | whole-file decode into RAM; sync decode stalls compile | M–L — Stream source, read-ahead thread, SPSC rings |
| Loop region/click/count-in | clip-source looping only | S each — executor loop region (declicked), click from tempo map |
| MIDI | zero anywhere | L via plugins; M via built-in instrument detour |
| Time-stretch | readiness enums only | M offline varispeed first; RT stretch non-goal |
| Musical time | pulse owns tempo (correct) | S — compile helper; never teach the executor BPM |
| Device lifecycle | negotiation + enumeration; no hot-swap/rate-change recovery | S–M |

## Operator correction (2026-06-11, after phase two)

This assessment's mixer framing ("busses+sends", "flat lanes→master")
leaked traditional-DAW vocabulary that does not match Loophole's model
(b04/b06/b08): faders and pans ARE Nodes, sends are Nodes, returns are just
Chains, free-standing Chains cover the bus use case without a bus type, and
pre/post-fader is topology. The correct gap statement: pulse's compile is a
flat interim projection bypassing the Chain/Node graph; the work is
chain-graph lowering (Nodes → Stages 1:1, sends → edges, fader → stage
gain, pan → edge matrix), specified in chorus g11.007 with the model audit
at chorus/research/2026-06-11-pulse-chain-model-audit.md. The engine's
Stage/Edge/matrix vocabulary needs nothing new.

## Operator direction adopted (2026-06-11, after this assessment)

Channel-format freedom (chorus a14): the mix graph is format-typed; nodes
up/downmix at will; spatial control is an N×M matrix primitive (pan = its
stereo special case); stereo collapse happens only at the hardware stage
when the device offers nothing wider. g10.010/013 amended accordingly —
edge formats and the matrix primitive land with the first schedule compile,
not as a retrofit.

## Sequencing (one line)

Graph plans → identity → parameter fast path build the spine; DSP kit and
metering fill it; DAC-time honesty and WYSIWYG bounce make time and export
truthful; recording then streaming are the product unlocks; runtime endgame
cashes the demolition dividend; MIDI/PDC/stretch wait for their
prerequisites. Proof infrastructure (golden renders, envelope property
tests, fake clocked backend for CI soak, xrun injection) lands distributed
across the packets, not as an afterthought.
