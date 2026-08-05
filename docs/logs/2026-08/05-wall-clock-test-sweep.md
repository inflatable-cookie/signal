# Wall-Clock Test Sweep

Status: complete
Created: 2026-08-05
Scope: the rest of the wall-clock test family, missed by the first pass

## The First Pass Was Incomplete

Gating the soak tests moved CI past the plugin bridge, and it then failed in
`signal-hardware`:

```
capture_session_records_the_fake_tone_to_wav
    expected ≈48000 frames, got 19712
capture_session_skips_initial_frames_before_writing
    expected ≈23_500 written frames, got 8992
```

Same mechanism, missed because the first sweep only searched
`crates/*/tests/*.rs`. These live in `#[cfg(test)]` modules inside `src/`.

A proper sweep parses every `#[test]` body under `crates/` for `thread::sleep`.
It finds sixteen, of which nine were still open.

## What Each One Actually Claimed

Expressing every floor as a fraction of real-time makes the risk order obvious:

| test | sleep | floor | implied throughput |
| --- | --- | --- | --- |
| `ticks_the_callback_at_block_cadence_and_honours_the_spec` | `100ms` | `10` blocks | `53%` |
| `ticks_the_capture_callback_with_a_tone_at_block_cadence` | `100ms` | `10` blocks | `53%` |
| `capture_session_records_the_fake_tone_to_wav` | `1000ms` | `24_000` | `50%` |
| `capture_session_skips_initial_frames_before_writing` | `500ms` | `12_000` | `50%` |
| `capture_with_monitor_tees_input_to_both_wav_and_sink` | `300ms` | `4_800` | `33%` |
| `gated_capture_waits_for_common_activation_but_keeps_monitoring` | `250ms` | `2_400` | `20%` |
| `monitor_session_duplicates_mono_input_to_stereo` | `100ms` | `256` | `5%` |

CI measured `41%` and `38%` on the two that failed. The two `53%` cadence tests
were next, and would have failed on the following run.

## Two Different Fixes

Gating everything would have been consistent and wrong: several of these tests
carry real correctness claims that CI should keep checking. The claim being made
decides the fix.

**Poll with a deadline** where the claim is liveness. The two cadence tests and
the capture allocation test now wait for ten blocks with a five-second deadline
instead of sleeping a fixed span. A slow host waits longer and still passes. The
cadence tests keep their upper bound, derived from time actually waited rather
than a fixed constant, so "ticked too fast" is still caught.

`capture_callback_path_allocates_nothing` comes *out* of the soak lane on this
basis. Allocation-freedom on the capture callback holds at any speed and is a
real-time contract worth checking on every run; only its block count was
load-dependent.

**Lower the floor** where the claim lives in the content assertions. The two
failing capture tests already prove what they care about by reading the WAV
back: the skip test reproduces the tone's phase and asserts the first written
sample equals the tone at frame `480`; the tone test checks RMS against
`0.354` and the zero-crossing rate against `440 Hz`. The frame-count range only
added "the host kept up". Floors drop to a liveness minimum, upper bounds stay,
because those still catch over-capture.

`monitor_session_duplicates_mono_input_to_stereo` is left alone at a `5%` floor,
a `37×` margin.

## Not Reproduced Locally

Worth stating plainly: this could not be reproduced on this machine. Saturating
all eighteen cores with busy loops did not fail the *old* code across three
runs, so that load model is not representative — macOS does not let normal
priority work starve these callback threads the way a shared runner does.

The fix therefore rests on construction rather than on a reproduction: the
throughput dependency is removed, not tuned. Floors that remain are `1%` to
`10%` of real-time instead of `50%`. CI is the only real test, and it is the
next step.

## A21 Was Measuring Noise

The soak gate then failed on `fake_clocked_soak`, which is `A21`. The assertion
was that after the injected starvation ends, the xrun counter grows by at most
one over a `500ms` window.

Measured rather than tuned. Over a `1500ms` window on an idle machine, the
recovered phase accrues between `2` and `8` xruns per `~281` callbacks across
six runs. The deliberately injected starvation is one stall per `32` callbacks,
which over the same window is `~8.8`.

The noise floor and the signal are the same magnitude. No threshold separates
"recovered" from "starved" here, because `FakeClockedBackend`'s own cadence
jitter produces xruns at nearly the rate the test injects on purpose. The
assertion never discriminated; it passed by luck and failed when the luck ran
out.

It is removed rather than loosened. Tuning the constant until it stops failing
would leave a test that cannot fail for the reason it claims to. The starvation
half of the claim still stands — the counter is asserted to have moved at all —
and playback advancement after recovery is still checked, which is robust.
Measuring recovery properly needs a clock with a tighter jitter floor than the
fake backend has.

`A21` is closed as unmeasurable-by-this-harness rather than fixed.

## Startup Budget Versus Latency Budget

CI then failed once more, on
`lv2_child_processes_blocks_and_killed_child_bypasses_within_budget`, at a `5s`
deadline for a spawned sandbox child to answer its first request.

Not the same defect as the rest of this sweep, and it matters that the two are
told apart. `signal-plugin-sandbox` `tests/plugin_hosting.rs` carries two kinds
of timing assertion:

- One **startup deadline**, guarding "did the child ever answer at all". The
  first request waits on a real process spawn plus a plugin `dlopen`, which on
  cold shared infrastructure legitimately takes seconds. A `5s` bound there
  measures the runner, not the bridge. Raised to `60s`, named, and documented.
  It still catches a hang, which is all it was ever for.
- Four **bypass budgets** at `<20ms` and one at `<2ms`, guarding "a dead child
  must not block the audio thread". Those are the product contract, not
  scaffolding. They are left exactly as they are.

The distinction is the point. Loosening a latency budget because it might flake
would discard the claim the test exists to make; loosening a startup deadline
discards nothing, because the assertion's content is "not never", not "fast".

The bypass budgets did pass this run. They remain a residual risk on shared
infrastructure and are worth watching, but they should not be pre-emptively
weakened.

## Twelve Spinning Children On Three Cores

The next CI run failed three tests in `plugin_hosting.rs`, and they were one
cause rather than three:

```
fixture_plugin_processes_a_chain_insert_through_the_real_engine_offline_render
    sample 7168: wet 0.5 vs dry 0.5 * 0.5
lv2_child_processes_blocks_and_killed_child_bypasses_within_budget
    child never answered a process request within 60s
sandboxed_fixture_editor_opens_over_the_wire_while_audio_stays_byte_exact
    child never answered a process request within 60s
```

The first is the same failure wearing different clothes: a missed response
bypasses and leaves the scratch untouched, so the insert never applies and wet
equals dry.

`60s` is not slowness. The `5s -> 60s` raise had already established that, and
the child still never answered. Spawn, startup receipts and plugin load all
succeed — the asserts covering those pass — so the child is alive and only the
audio round trip fails.

The cause is the test harness, not the bridge. The sandbox child runs a
hot-spinning audio thread, and this binary holds twelve tests that each spawn
one. Run in parallel that is twelve spinning children plus twelve spinning
parents. On this laptop's eighteen cores it fit; on a GitHub runner's three or
four it did not, and the children could not get enough CPU to answer inside
their budget.

Cargo already runs test *binaries* sequentially, so all of this contention was
self-inflicted within the one binary. The eleven child-spawning tests now take a
mutex and run one at a time. Serialised: `12 passed` in `1.13s` against `0.39s`
parallel, and `0` failures in `15` consecutive runs.

Nothing about the timing budgets moved for this. The bypass budgets are still
`<20ms`.

## Next Task

Dispatch CI. If green, tag `v0.1.0`.
