# God-File Split: LV2 turtle/hosting + hardware capture

Status: complete
Created: 2026-08-07
Scope: `signal-plugin-lv2` turtle + hosting; `signal-hardware` capture

## Baseline

After prod-critical clearance: critical=13 (tests/fixtures only). Next high
band included LV2 `turtle.rs` (654 code), LV2 `hosting.rs` (468 code),
hardware `capture.rs` (639 code).

## What Changed

### `signal-plugin-lv2` `turtle`

→ `turtle/{document,parser,tests}`

### `signal-plugin-lv2` `hosting`

→ `hosting/{support,instance,process}`

### `signal-hardware` `capture`

→ `capture/{monitor,session,stopped,tests}`

Move-only. Public crate re-exports unchanged.

## After

Cleared those three high-band production files from the top of the scan.

## Validation

- `cargo fmt` / `clippy -D warnings` on touched crates
- `cargo test -p signal-plugin-lv2 --lib` — 15 passed
- `cargo test -p signal-hardware --lib capture::` — 6 passed

## Next Task

Continue high-band prod shrinkage (stretch engine/backend/resumable,
render-plane offline artifact, clap discovery) or stop for review.
