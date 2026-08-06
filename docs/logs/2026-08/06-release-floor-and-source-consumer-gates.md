# Floor And Source-Consumer Gates, And A Hole In The Reference Assertion

Status: complete
Created: 2026-08-06
Scope: the two release gates proven by swallowtail `v0.1.1`, adopted in
longhorn's single-toolchain shape, plus the `1.95` Rust floor

## The Floor Was Declared, Not Verified

`c375f410` raised `rust-version` to `1.95` — `cfg_select` stabilised there and
libsqlite3-sys needs it via longhorn's rusqlite, and underlay had independently
gone to `1.94`, so `1.95` is the measured portfolio ceiling. The manifest said
so; nothing checked.

First run of `scripts/check-release-floor.sh` found a real violation:

```
error: this `if` can be collapsed into the outer `match`
  --> crates/signal-dsp-stretch/src/creative_direct_renewal_dream/synthesis.rs:189:13
  = note: `-D clippy::collapsible-match` implied by `-D warnings`
```

`1.95`'s clippy denies a `collapsible_match` that `1.97.1`'s does not. The
workspace has been clippy-clean at the pinned toolchain throughout, which is a
weaker claim than it reads as: two toolchains are two different demands, and the
floor is a promise to consumers who will use neither of ours. Fixed as a match
guard, which both accept.

After that, clippy and the full suite pass at `1.95.0`.

## Clippy And Tests, Not `cargo check`

Copied deliberately from longhorn, where the floor violation appeared only under
`--all-targets`: a dev-dependency pulled the feature that compiled the offending
build script, and a bare `cargo check` passed and gave false confidence. Signal
has no per-crate MSRV override, so one pass suffices; the two-pass shape stays
documented in the script for whenever one is added.

## The Source-Consumer Gate Found Something About The Gate

`scripts/verify-source-consumer.sh` builds a throwaway crate depending on this
tree by `git = "file://…", rev = "…"` — `23` probe crates, being every crate
`finch`, `soundcheck`, `soundcheck-library` and `loophole` actually name, plus
`signal-dsp` for its domain — then asserts via `cargo metadata` that every
`signal-*` package resolves from `git+file://…#<commit>`.

It passed on the first run. Then the assertion itself was tested, and it should
not have.

The reference filter is:

```jq
select((.name | startswith("signal-")) and .source != null)
```

A path-resolved package has `source: null`, so that clause **excludes** the leak
being hunted rather than failing on it. Only the `>= expected` count remains,
and signal has `28` workspace crates against `23` probes — five crates of slack.

Measured, by pointing one probe at a path instead of a git rev:

| assertion | result |
| --- | --- |
| reference (`.source != null`) | **passed**, printing "external source consumer passed" |
| tightened (select by name) | failed |

`cargo check` succeeded in both cases, which is the whole point: that is exactly
what a consumer with sibling checkouts sees.

The fix selects by name and excludes only the throwaway consumer package —
which is itself `signal-`-prefixed and legitimately sourceless, and is why the
reference reached for `.source != null` in the first place. The count is kept as
a guard against the probe list silently shrinking.

This is a portfolio finding, not a signal one. It bites wherever a repo has more
crates than probes, which is the normal case as a workspace grows.

## Neither Gate Can Run In CI

`floor` needs a second rustup toolchain installed beside the pinned one.
`source-consumer` builds a throwaway crate against a git source of the working
tree. Both join `soak` as claims made only at release time — the pattern the
runbook now cites, and the reason `config/release.toml` is worth more than a
mirror of the CI workflow.

Wired as `effigy release:floor`, `effigy release:source-consumer`, and the
aggregate `effigy release:gates`, so a finding can be chased without driving a
whole release.

## The Snapshot Path Matters More Than It Looks

The script snapshots a dirty tree into a throwaway git repo rather than
refusing. A gate that only runs once everything is committed is a gate that runs
after the point where acting on a finding is cheap. Both real runs here were
snapshots.

## Note On `rust-toolchain.toml`

Its comment claimed the `1.90` floor was declared and "nothing currently
verifies it". Both halves were stale within a day. Now it names `1.95`, points
at the gate that verifies it, and records that the pinned and floor toolchains
deny different lints so code must satisfy both.

## Next Task

Cut `v0.1.1` if the full gate set is green. The source-consumer gate found
nothing about `v0.1.0`'s consumability, so the patch version carries the
`1.95` floor, the `collapsible_match` fix, and the gates themselves.
