# A19 - Use After Unmap In The Shm Round-Trip Test

Status: complete
Created: 2026-08-05
Scope: finding `A19`, closed with a mechanism

## How It Surfaced

The `test` release gate failed with a segfault, not an assertion:

```
process didn't exit successfully: signal_plugin_bridge-cf6261bae37c786c
  (signal: 11, SIGSEGV: invalid memory reference)
```

Three immediate reruns were clean and macOS produced no crash report. Twelve
runs found it twice, both times naming
`shm::tests::served_request_round_trips_through_the_region` — which is `A19`,
carried since `g10.038` as "no mechanism found".

## The Mechanism

`A19` presented as an intermittent assertion failure *and* an intermittent
segfault. It was both, from one cause.

The test builds `child_view` as a raw pointer into `region`'s shared mapping and
moves it into a spawned server thread that emulates the sandbox child. The
client then retries `handle.process` and asserts the round trip. `server.join()`
sat *after* those assertions.

The retry loop was bounded by iteration count:

```rust
for _ in 0..200 {
    if handle.process(&mut scratch, 32, 2) { processed = true; break; }
}
assert!(processed, "server should have answered within retries");
```

Two hundred iterations of this thread is not a bound on the other thread being
scheduled. It is an assumption about host contention, and under a loaded
machine it loses.

When it loses, `assert!(processed, ...)` panics. Unwinding drops `region`, which
unmaps the backing memory while the server thread is still spinning on
`child_view.request_seq()`. The thread dereferences unmapped memory and the
whole test binary dies with `SIGSEGV` — taking the assertion message with it,
which is why the failure looked like two unrelated flakes.

## The Fix

Both halves are corrected, because either alone leaves a real defect.

The retry loop is bounded by a five-second deadline rather than an iteration
count, so a slow host waits longer instead of failing.

`server.join()` moves *ahead* of every assertion. The mapping now cannot be
unmapped while the thread that reads it is alive, whatever any assertion does.
The join result is captured and asserted after `processed`, so a genuine server
failure is still reported.

The server thread's own deadline goes from `2s` to `10s`, longer than the
client's `5s`, so the client decides the outcome. A shorter server deadline
would make that thread panic first under contention and report "server never saw
a request" when the real answer is "the host was busy".

## Measurement

- Before: `2` failures in `12` runs of `signal-plugin-bridge --lib`, plus the
  gate segfault.
- After: `0` failures in `15` runs.

## Findings State

`A19` closed. `A22` closed earlier today. `A20` and `A21` are relocated to the
soak lane rather than fixed — their assertions are host-speed claims and are
only meaningful there. `A18`, the low-mid pops on ticks, remains open and needs
a direct transient probe.

## Next Task

Dispatch CI. If green, tag `v0.1.0`.
