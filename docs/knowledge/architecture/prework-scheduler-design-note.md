# Prework Scheduler Design Note (post-deletion record)

Status: historical record. The anticipative prework scheduler was deleted in
g10.020 together with the engine-block simulation it was welded to. This note
preserves the policy vocabulary worth re-deriving when anticipative rendering
is scheduled against the render plane. Full implementation: git history before
g10.020 (`crates/signal-runtime/src/runtime_prework_*`,
`runtime_prework_forecast/`, `runtime_prework_state/`,
`interfaces/prework_forecast_family.rs`).

## What it was

A control-plane service that pre-computed audio buffers for future block
sequences ("prework") and admitted them into a bounded cache the realtime
dispatch could consume instead of rendering live. Admission validated the
cached entry against the transport/parameter state it was prepared under;
any divergence retired the entry with a typed reason.

## Policy vocabulary worth keeping

- **Backlog classes** (`Immediate` / `NearTerm` / `Deferred`): each pending
  prework target carried an urgency class; service cycles drained the queue
  up to a maximum class, so under pressure only near-deadline targets were
  prepared.
- **Pressure tiers** (`Normal` / `Elevated` / `Critical`): the realtime side
  signalled load. Normal serviced the full window; Elevated shrank cycles and
  budget to 1 and tightened the admissible backlog class; Critical yielded
  immediately (zero budget). Yields and starvation were recorded as explicit
  service-state transitions (`Idle/Pending/Servicing/Yielding/Paused/Starved`).
- **Budget gating**: service ran as `cycles x budget_per_cycle` prepared
  blocks per invocation, with a multicore scale factor widening the budget
  when extra cores were available. Plugin-activity and transport-activity
  gates forced a yield regardless of budget.
- **Semantic policy** (`Balanced` / `LatencyFocused` / `PluginConstrained`):
  how Elevated pressure degraded — latency-focused kept a slightly wider
  budget, plugin-constrained clamped to Immediate-only.
- **Forecast profiles and modes**: `Disabled / RuntimeRoleDefault /
  ExplicitProfile / RawPolicyOverride`, with `Local` and `Server` profiles
  defining target window size (blocks ahead) and prepare budget per cycle.
- **Invalidation/retirement reasons**: a closed enum of ~18 causes
  (reconfigure, stop, plan change, graph change, transport start/stop/seek/
  tempo/loop-wrap, parameter batch, input-signature change, epoch/sequence
  expiry, supersession, window revision, queue capacity). Freshness was a
  derived state (`Fresh / Expiring / Exhausted / Invalidated`) relative to
  the playhead.

## Re-derivation framing

From the post-demolition assessment: pre-render plan regions into sample
buffers ahead of the playhead; the render plane's Samples/Stream sources
already know how to play them. The scheduler's worthwhile ideas were never
the simulated engine blocks — they were the admission/invalidation contract
(typed reasons, freshness relative to the playhead) and the pressure/budget
ladder for stealing time from the realtime thread safely. A future
anticipative renderer should re-derive those against real render-plane
sources rather than resurrecting this code.
