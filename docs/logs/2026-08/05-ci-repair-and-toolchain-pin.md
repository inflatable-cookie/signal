# CI Repair And Toolchain Pin

Status: complete
Created: 2026-08-05
Scope: CI workflow, toolchain determinism, new finding `A22`

## CI Had Never Passed

CI has failed on every push since it was added. The toolchain step ran:

```
rustup toolchain install stable --profile minimal --component rustfmt clippy
```

`rustup` takes one value after `--component`, so `clippy` was parsed as a
toolchain name and the step exited `1`. Runs lasted `3` to `12` seconds. Nothing
was ever built, linted, or tested. The `0.1.0` release commit inherited the same
failure.

Components now live in `rust-toolchain.toml`, which already pinned the channel,
so there is one declaration rather than two that drift.

## CI Did Not Run The Gates

Even repaired, the workflow ran a different set of commands from
`config/release.toml`: no `--all-features`, no features-off lint pass, no
missing-docs check. CI could have gone green while a release gate failed, which
makes the green meaningless as release evidence. The workflow now mirrors the
cargo-backed gates.

`validate` and `docs` are not mirrored. They run through effigy, which is not
installed on the runner. The workflow states that rather than implying coverage
it does not have.

## The Toolchain Was Floating

With CI finally building, clippy failed on three `unneeded_wildcard_pattern`
sites in `signal-runtime` that the local gates had passed. The gates were run on
`1.96.0` from May; the runner installed `1.97.1`. `channel = "stable"` floats,
so "the gates pass" and "CI passes" were claims about different compilers.

The channel is pinned to `1.97.1`. The three redundant `detail: _` patterns —
each sitting beside a `..` that already covers them — are removed.

This is not the MSRV. `rust-version` declares the `1.90` floor and nothing
verifies it: CI builds only on the pinned version. Worth closing separately.

> Closed 2026-08-06. The floor is now `1.95` and
> `2026-08/06-release-floor-and-source-consumer-gates.md` records the gate that
> verifies it — which found a real violation on its first run, so "nothing
> verifies it" was not a theoretical gap.

## New Finding `A22`

`signal-plugin-sandbox` `tests/plugin_hosting.rs` fails intermittently under
parallel load, in the `A20` and `A21` class but more severe.

Observed on unmodified `f4b32b1a`, so it predates this work. Run alone it passes
every time in `0.06s`. Run as a full binary on an idle machine it passed ten
consecutive times in about `0.5s`. Under concurrent cargo activity it failed
`2/12`, `5/12`, `6/12`, and `7/12` across four runs, with wall time rising from
`0.46s` to `5.9s`.

Failures cluster on the tests that spawn real child processes hosting real
system audio units — `real_child_*`, `au_child_*`, `au_wire_*`. Those assert
timing budgets, so the mechanism is consistent with the rest of the class:
the budget is missed under contention rather than any logic being wrong. No
leaked child processes or shared memory segments were found after a failing run.

This matters for the release specifically, because `cargo test --workspace` is
the loaded condition and it is what both the `test` gate and CI run. A flaky
gate is a gate that can be retried until green, which is not a gate.

Untriaged alongside `A18`, `A19`, `A20`, `A21`.

## Next Task

Watch CI on the pinned toolchain. If `A22` surfaces there, it blocks the tag
until the timing budgets are either made load-independent or the affected tests
are marked and excluded honestly.
