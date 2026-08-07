# God-File Split: VST3 instance + AU hosting

Status: complete
Created: 2026-08-07
Scope: `signal-plugin-vst3` hosting instance; `signal-plugin-au` hosting

## Baseline

After prior hosting/broker batch: critical=17. Next prod criticals included
VST3 `hosting/instance.rs` (~752 code) and AU `hosting.rs` (~1221 code).

## What Changed

### `signal-plugin-vst3` `hosting/instance`

→ `hosting/instance/`:

- `layout` — port/bus layout + arrangement helpers
- `controller` — edit-controller acquire/connect/inventory
- `mod` — `Vst3HostedInstance` lifecycle

### `signal-plugin-au` `hosting`

→ `hosting/`:

- `types` — error, registry sentinel, FourCC load keys
- `ffi` — AudioToolbox/CoreFoundation (macOS)
- `instance` — `AuHostedInstance` + property helpers
- `process` — pull-model `AuProcessSession`
- `tests`

Move-only. Public crate re-exports unchanged.

## After

`effigy scan god-files`: critical=15.

Cleared production criticals:

- VST3 `hosting/instance.rs`
- AU `hosting.rs`

## Validation

- `cargo fmt` / `clippy -D warnings` on touched crates
- `cargo test -p signal-plugin-vst3 --lib` — 26 passed
- `cargo test -p signal-plugin-au --lib` — 13 passed

## Next Task

Next prod criticals: VST3 `introspection.rs`, then `creative.rs` /
`phase_vocoder.rs` — or stop for operator review / commit.
