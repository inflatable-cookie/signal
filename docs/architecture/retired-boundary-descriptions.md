# Retired Boundary Descriptions

Status: informative
Created: 2026-06-11

`signal-supervisor-tools` (deleted in g10.004) carried ~9.8k LoC of hardcoded
boundary-rationale prose rendered through `--describe-*` CLI flags, plus an
acceptance-lane manifest tree that asserted that prose against itself. The
still-true content reduces to a few sentences; everything else restated crate
purposes that the crates' own docs already state.

What was worth keeping:

- **Boundary principle** — runtime truth is exported through shared,
  consumer-facing report types rather than adapter-private state; host
  surfaces consume the same public exports the tests do. This survives as
  the actual design of `signal-runtime`'s public API and is enforced by its
  `public_*_boundary_*` tests, not by descriptions.
- **Validation posture** — the canonical validation commands are the
  workspace's own: `cargo build --workspace`, `cargo test --workspace`, and
  `effigy validate` / `effigy qa`. Per-boundary "validation step" listings
  duplicated this per domain.
- **Platform split** — macOS (AU/CoreAudio) and Linux (LV2/PipeWire/ALSA)
  boundary work was descriptive only in the deleted supervisor prose; the live
  platform story starts at `signal-hardware-cpal` (negotiated output streams +
  device enumeration, `g10.003`) and continues through the shipped plugin-hosting
  adapters. Remaining deferred platform/device depth lives in
  `docs/roadmaps/backlog/post-g10-rebuild-on-demand.md`.

`signal-host-server` (deleted in the same packet) contained no serving
machinery — it was an in-process copy of `signal-host-local` plus the LV2
adapter wiring, with a mirrored test suite. Nothing was relocated from it;
an eventual engine server starts from the `signal-ipc` shared-memory broker
per the rebuild backlog.

Full text of both crates remains in git history (pre-g10.004 commits).
