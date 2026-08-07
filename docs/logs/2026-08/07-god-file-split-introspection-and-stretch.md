# God-File Split: VST3 introspection + stretch creative/phase_vocoder

Status: complete
Created: 2026-08-07
Scope: `signal-plugin-vst3` introspection; `signal-dsp-stretch` creative + phase_vocoder

## Baseline

After prior instance/AU batch: critical=15. Next prod criticals included
VST3 `introspection.rs` (~1263 code), `creative.rs` (~979 code),
`phase_vocoder.rs` (~705 code).

## What Changed

### `signal-plugin-vst3` `introspection`

→ `introspection/`:

- `types` — metadata/factory COM layouts
- `macos_bundle` — CFBundle factory load (macOS)
- `paths` — bundle info + module path helpers
- `derive` — IO/feature/vendor derivation
- `factory` — in-process factory class enumeration
- `scan_helper` — out-of-process helper + tests
- `snapshot` — bundle snapshot assembly

### `signal-dsp-stretch` `creative` / `phase_vocoder`

→ sibling modules with tests extracted:

- `creative/{mod,tests}`
- `phase_vocoder/{mod,tests}`

Move-only. Public crate re-exports unchanged.

## After

`effigy scan god-files`: critical=13.

Cleared production criticals:

- VST3 `introspection.rs`
- `creative.rs`
- `phase_vocoder.rs`

Residue: large test modules may still score critical (expected).

## Validation

- `cargo fmt` / `clippy -D warnings` on touched crates
- `cargo test -p signal-plugin-vst3 --lib` — 26 passed
- `cargo test -p signal-dsp-stretch --lib creative::` — 15 passed
- `cargo test -p signal-dsp-stretch --lib phase_vocoder::` — 16 passed

## Next Task

No remaining production criticals under the usual filter (tests/fixtures/
demos/benchmarks excluded). Next options: shrink high-band prod files,
tackle remaining test criticals, or stop for operator review / commit.
