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

## `v0.1.0` Left Its Prepared State Behind

`effigy release prepare` for `0.1.1` refused to run:

```
release state file already exists: /Users/tom/Dev/projects/signal/.release-prepared.json
```

The file was the `v0.1.0` one — `previous_version: 0.0.0`, `prepared_at:
2026-08-05`, `prepared_head: b2d6cabd`. `execute` tagged and pushed `v0.1.0` and
did not clear it, so the first release silently blocked the second.

Verified obsolete before removing rather than removing on sight: `v0.1.0` is
tagged locally at `e52721a9`, present on `origin`, and `b2d6cabd` is an ancestor
of `HEAD`. The file is gitignored local state describing a completed release.

`effigy release resume` is the documented recovery entrypoint but is
interactive; it hung waiting for a menu selection under a non-interactive shell
and had to be killed. Worth knowing before reaching for it in a script.

Also confirmed while there: prepare's `Files Modified` is now `Cargo.toml` and
`CHANGELOG.md` only. The `v0.1.0` state file lists `Cargo.lock` as well, which
is the `sync-files` behaviour removed after it bumped ~40 dependencies after the
gates had run. The removal holds.

## `--locked` Made The Floor Gate Unrunnable In The Release Flow

Copied straight from the reference scripts, and it deadlocks:

```
error: cannot update the lock file .../Cargo.lock because --locked was passed
```

`effigy release prepare` bumps `workspace.package.version` and *then* runs the
gates. At gate time the manifest says `0.1.1` and Cargo.lock still says `0.1.0`,
so `--locked` refuses. Effigy also requires `--check-gates` when gates are
configured, so there is no way to gate the tree at the version being released
without resolving this.

Two failure modes hid behind it. A failed prepare is not atomic — it left
`Cargo.toml` and `CHANGELOG.md` mutated while reporting `Prepared: no` and
`State file: not written`. And had the gate simply dropped `--locked`, the tag
would have carried a manifest at `0.1.1` beside a lockfile at `0.1.0`, which is
exactly what breaks a consumer building the tag with `--locked`.

The floor gate now syncs the workspace lock before gating. `cargo update -w`
touches workspace members only — not `cargo generate-lockfile`, which is the
`sync-files` behaviour removed after it bumped ~40 third-party crates *after*
the gates had run on the `0.1.0` prepare. Measured on this bump: `28` changed
lines, every one a signal crate going `0.1.0 -> 0.1.1`, nothing third-party.

The sync is verified, not trusted. Every changed line must be a `version` line
and every added one must equal the workspace version, so a third-party bump —
also a version line — is still refused. Tested in all three directions:

| case | result |
| --- | --- |
| lock already in sync | silent, accepted |
| workspace `0.1.0 -> 0.1.1` | accepted, reported |
| fabricated `crossbeam-epoch 0.9.18 -> 0.9.99` | refused, offending diff printed |

The same conflict is latent in swallowtail's and longhorn's floor scripts, which
also pass `--locked`. It only bites once a floor gate is wired into
`[release.gates]` rather than run by hand.

## A Note On The Version Number

`0.1.1` carries an MSRV raise from `1.90` to `1.95`, which stops the build for
any consumer below `1.95` — conventionally a minor bump rather than a patch.
Recorded rather than acted on: the whole portfolio is moving to `1.95` in
lockstep, every consumer is in-tree, and the patch version was the one
specified.

## Next Task

Cut `v0.1.1` if the full gate set is green. The source-consumer gate found
nothing about `v0.1.0`'s consumability, so the patch version carries the
`1.95` floor, the `collapsible_match` fix, and the gates themselves.
