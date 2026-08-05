# Soak Lane And Selective CI

Status: complete
Created: 2026-08-05
Scope: wall-clock test separation, CI triggers, findings `A20` and `A22`

## The Failure

The first CI run that got far enough to run tests failed on
`au_in_process_soak_processes_every_clocked_block_without_misses`:

```
expected sustained clocked callbacks, saw 70
```

The test sleeps `1200 ms` and asserts at least `100` callbacks, where the fake
clocked backend emits one roughly every `5.3 ms`. It is asserting that the host
sustained better than half of real-time throughput for over a second.

That is not a correctness property. It is a claim about the machine, and a
shared CI runner cannot honour it. Neither can a developer laptop running a
build in another window — which is exactly what `A20`, `A21` and `A22` were.

## What Changed

Seven tests move to an opt-in soak lane, gated on `SIGNAL_SOAK_TESTS=1`:

| test | crate |
| --- | --- |
| `capture_callback_path_allocates_nothing` | `signal-hardware` |
| `au_in_process_soak_processes_every_clocked_block_without_misses` | `signal-plugin-bridge` |
| `lv2_in_process_soak_processes_every_clocked_block_without_misses` | `signal-plugin-bridge` |
| `vst3_in_process_soak_processes_every_clocked_block_without_misses` | `signal-plugin-bridge` |
| `clocked_soak_advances_health_counters_and_meters` | `signal-render-plane` |
| `live_events_while_stopped_sound_and_hold_zero_alloc_under_clocked_load` | `signal-render-plane` |
| `callback_health_counters_advance_and_infer_xruns` | `signal-render-plane` |

Every one sleeps for a fixed wall-clock span and then asserts a minimum
callback count, or asserts zero xruns — which is the same claim stated
negatively. `callback_health_counters_advance_and_infer_xruns` is `A20`
directly.

They skip loudly, printing the variable that enables them, rather than silently
reporting `ok`. `effigy test:soak` runs them with `--test-threads=1`, because
running them beside the rest of the suite creates the contention that breaks
them. The lane takes about `35s` and is now the `soak` release gate, so the
throughput claim is still required before a tag — it just is not made by CI.

This is a real reduction in what the default suite proves. The throughput
claims still need making; they now get made in a place where the answer means
something, and the release lane should run `effigy test:soak` before a tag.

## A22 Was A Consequence

`A22` — `signal-plugin-sandbox` `tests/plugin_hosting.rs` failing under parallel
load — was caused by these same tests. They sleep for seconds while their
callback threads stay hot, which is exactly the contention that made the sandbox
timing budgets miss.

Measured rather than assumed. Before gating, the binary failed `2`, `5`, `6` and
`7` of `12` across four runs under concurrent cargo activity. After gating:

- `cargo test --workspace` three consecutive times: clean, all three.
- `plugin_hosting` binary ten consecutive times: `12 passed; 0 failed`, all ten.

`A22` is closed. `A20` is not fixed but relocated: it is one of the seven tests,
and it now runs in the soak lane where its zero-xrun assertion is meaningful.
`A21` is the same. `A18` and `A19` are untouched and remain open.

## Selective CI

CI ran on every push to `main`. The job builds the whole workspace twice under
clippy plus a full test run on `macos-latest`, which GitHub bills at ten times
the Linux rate, so every documentation commit was paying for a full native
build.

Triggers are now `workflow_dispatch`, `pull_request`, and `push` on `v*` tags.
The release flow becomes: commit to `main`, dispatch the workflow, wait for
green, then tag.

`macos-latest` is not negotiable. The AU and VST3 host adapters and the
CoreAudio hardware layer only build there.

The cost of this is real and worth stating: `main` no longer gets continuous
verification between releases. A regression can sit on `main` until someone
dispatches the workflow or opens a pull request.

## Next Task

Dispatch CI on the pinned toolchain with the soak lane gated. If green, tag
`v0.1.0`.
