# Consuming Signal

Status: current
Created: 2026-08-05
Scope: how downstream repositories depend on Signal

Signal's crates are not published to crates.io. Consumers pin a git tag and
reference the crates by URL, so a release is a tagged commit rather than a
registry upload. Local development against the Signal working tree is opt-in
per machine and never committed.

This is the canonical runbook. It is written here rather than left in chat
because the lockfile rule below is easy to get wrong and was in fact got wrong
during the `v0.1.0` migration.

## The committed form

Every consumer's manifest carries the released address:

```toml
signal-plugin-vst3 = { git = "ssh://git@github.com/inflatable-cookie/signal.git", tag = "v0.1.0" }
```

A clean checkout with no sibling repositories on disk builds from this alone.
Colleagues and CI have no override file, so they resolve the tag from git and
there is nothing to coordinate.

The URL is the `ssh://git@github.com/...` form throughout, matching the
convention already in place. `https://github.com/...` works equally well, but
the two are different keys as far as `[patch]` is concerned — the table key
must match the dependency URL character for character, so a repository must
not mix them.

## The local override

`scripts/signal-link.sh` writes a gitignored `.cargo/config.toml` at the
consumer root:

```toml
[patch."ssh://git@github.com/inflatable-cookie/signal.git"]
signal-plugin-vst3 = { path = "../signal/crates/signal-plugin-vst3" }
# ...one line per crate in the resolved graph
```

`scripts/signal-unlink.sh` removes it. Config-level `[patch]` needs no manifest
edit, so `git status` stays clean in both states. Verify an override is live
with `cargo tree -p signal-plugin-vst3`, which shows a path rather than the git
URL.

## Patch the resolved graph, not the direct dependencies

A partial patch list is worse than no list. The named crates resolve to the
working tree while the rest come from the tag, so one build holds two copies of
`signal-primitives` and the trait implementations stop matching, with errors
that point nowhere useful.

The gap is wide in practice. Measured during the `v0.1.0` migration:

| consumer | direct deps | resolved graph |
| --- | --- | --- |
| `finch` | `6` | `8` |
| `soundcheck` | `5` | `16` |
| `soundcheck-library` | `6` | `7` |
| `loophole` | `17` | `23` |
| `monkey` | `2` | `2` |
| `jetstream` | `4` | `6` |

`jetstream` was already carrying this bug: its link script patched four crates
against a six-crate graph, so `signal-dsp-resample` and `signal-dsp-stretch`
kept resolving to the pinned revision while their siblings resolved to the
working tree.

Regenerate a list with `cargo metadata`, taking every package whose source is
the Signal remote, across every workspace in the repository.

## Never commit a lockfile from a patched build

This is the one real cost of the mechanism. Building while linked rewrites the
lock entries for patched crates to the path source, which drops the git address
entirely:

```
 [[package]]
 name = "signal-analysis"
 version = "0.1.0"
-source = "git+ssh://git@github.com/inflatable-cookie/signal.git?tag=v0.1.0#e52721a9"
```

A colleague or CI resolving from that lockfile has no address to fetch. Removing
the patch and rebuilding reverts it.

`finch` shipped exactly this in the migration commit and needed a follow-up.
So it is checked rather than remembered: `scripts/signal-check-lock.sh` fails
if any `signal-*` entry in a committed lockfile lacks a `source` line. It is
verified against a genuinely patched lockfile, not just a clean one.

To commit a lockfile change: run `scripts/signal-unlink.sh`, rebuild, commit,
then re-link.

## Per-repository or global

A single `[patch]` block in `~/.cargo/config.toml` would cover every consumer
at once and save the bookkeeping. Per-repository files are used instead because
they keep the toggle: "does the tag itself build" is answerable per consumer by
running `signal-unlink.sh`, without commenting out a block that every other
repository also depends on. That question is worth keeping cheap — the `v0.1.0`
migration verified all six consumers in both states, and the unlinked half is
the one that proves the release.

## Current consumers

`finch`, `soundcheck`, `soundcheck-library`, `loophole` (three workspaces:
`pulse`, `spark`, `aura/src-tauri`), `monkey`, `jetstream`.

`keepsake` is C++ and has no Signal dependency.

## Next Task

None. Update the table above when a consumer's dependency set changes.
