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
signal-plugin-vst3 = { git = "ssh://git@github.com/inflatable-cookie/signal.git", tag = "v0.1.1" }
```

A clean checkout with no sibling repositories on disk builds from this alone.
That is no longer taken on trust: `effigy release:source-consumer` builds exactly
such a checkout against the release commit at every release and asserts every
`signal-*` package resolves from a git source rather than a path.

**Rust floor: `1.95` from `v0.1.1`** (`1.90` at `v0.1.0`). Verified at release
time by `effigy release:floor`, which runs clippy and the full suite under that
toolchain -- not `cargo check`. A consumer below `1.95` should stay on `v0.1.0`.
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
if any `signal-*` entry lacks a `source` line.

It inspects the **staged** content when there is any, otherwise `HEAD`. It
deliberately ignores the working tree, because while linked a dirty lockfile is
the normal state — flagging that would make the check noise rather than signal.
The first version got this wrong and reported every linked working copy.

Verified in both directions: it passes a clean lockfile and fails a genuinely
patched one, naming each affected crate and exiting `1`.

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

## Sandbox broker prebuilt contract

Stable Cargo does not give a dependent package access to another package's
binary. `signal-plugin-sandbox` is therefore **not** consumable as a normal
Cargo dependency for its executable. Depending on that package alone does not
build or path the broker.

The supported consumer boundary is a compatible **prebuilt** broker executable
supplied before startup:

| Variable | Role |
| --- | --- |
| `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND` | Absolute path to the broker executable (required to enable the broker path) |
| `SIGNAL_PLUGIN_SANDBOX_BROKER_ARGS` | Optional arguments (shell-style quoting) |
| `SIGNAL_PLUGIN_SANDBOX_BROKER_WORKDIR` | Optional child working directory |
| `SIGNAL_PLUGIN_SANDBOX_BROKER_READ_TIMEOUT_MS` | Optional receipt read timeout |

Rules:

- Set `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND` to a real executable before the
  consumer process starts. Missing configuration fails fast with an actionable
  diagnostic from `signal-runtime` (`SandboxBrokerClientSession::spawn_from_env`).
- Consumer startup must **not** invoke Cargo or build Signal source to obtain
  the broker. An on-demand `cargo build` / `cargo run` inside the first test or
  product boot is out of contract.
- Provisioning may build or retrieve the broker in an **explicit** prior step.
  In this repository: `effigy broker:provision` (or
  `bash scripts/provision-sandbox-broker.sh`) prints a host-local absolute path
  suitable for the env var. Optional inputs:
  `SIGNAL_BROKER_TARGET_DIR`, `SIGNAL_BROKER_PROFILE` (`debug` or `release`).
- The provisioned binary is for the current host and profile. Do not treat one
  machine's artifact as portable across OS or architecture.

Example:

```sh
export SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND="$(effigy broker:provision)"
# then start the consumer / test process that spawns the broker
```

Focused Signal proof of this boundary: `effigy broker:prove-prebuilt-contract`.

Decision record: `docs/triage/2026-09-01-sandbox-broker-prebuilt-contract.md`.
Earlier Cargo-dependency diagnosis:
`docs/logs/2026-08/31-papercuts-wave29-sandbox-broker-consumer-diagnosis.md`.

## Next Task

None. Update the consumer table when a dependency set changes; revisit broker
distribution only if Signal later chooses release-shipped assets or stable
Cargo artifact dependencies.
