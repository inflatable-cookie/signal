# God-File Split: VST3 hosting + CLAP hosting + sandbox broker

Status: complete
Created: 2026-08-07
Scope: `signal-plugin-vst3` hosting; `signal-plugin-clap` hosting; `signal-plugin-sandbox` broker

## Baseline

After prior god-file batches: critical=19. Next prod criticals included VST3
`hosting.rs` (~3k), CLAP `hosting.rs` (~1.7k), sandbox `broker.rs` (~1.5k).

## What Changed

### `signal-plugin-vst3` `hosting`

→ `hosting/`:

- `wire/` — COM/host wire co-located (`com`, `stream`, `host_application`,
  `module`, `parameters`, `events`)
- `instance` — `Vst3HostedInstance` lifecycle
- `process` — `Vst3ProcessSession` + bus helpers/tests

Public restart constants and instance/process re-exports unchanged at the
adapter boundary.

### `signal-plugin-clap` `hosting`

→ `hosting/`:

- `entry` — `ClapHostingError`, `LoadedClapEntry`
- `host` — `ClapHostShim` + host-extension callbacks
- `instance` — `ClapHostedInstance` lifecycle
- `process` — `ClapProcessSession`

### `signal-plugin-sandbox` `broker`

→ `broker/`:

- `types` — state/receipt/command + wire helpers
- `hosted` — format-selected instance wrappers
- `process` — serve loop + receipts
- `lifecycle` — load/activate/editor/processing commands
- `shm` — attach/run/teardown
- `tests`

Move-only. Child binary public surface unchanged
(`SandboxBrokerProcess`, receipt/state helpers).

## After

`effigy scan god-files`: critical=17.

Cleared production criticals:

- CLAP `hosting.rs`
- sandbox `broker.rs`
- VST3 `hosting.rs` monolith (residue: `hosting/instance.rs` still critical)

## Validation

- `cargo fmt` / `clippy -D warnings` on the three packages
- `cargo test -p signal-plugin-vst3 --lib` — 26 passed
- `cargo test -p signal-plugin-clap --lib` — 15 passed
- `cargo test -p signal-plugin-sandbox --bin signal-plugin-sandbox` — 11 passed

## Next Task

Next prod criticals: VST3 `hosting/instance.rs`, AU `hosting.rs`,
`introspection.rs`, then `creative.rs` / `phase_vocoder.rs` — or stop for
operator review / commit.
