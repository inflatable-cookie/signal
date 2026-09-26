# Release

Signal releases tagged Git commits. Crates are not published to crates.io.
Consumers pin the tag by Git URL; see the
[consumer runbook](../../reference/consuming-signal.md). The workspace version
in `Cargo.toml`, `CHANGELOG.md`, and the contract set define the claim.
`config/release.toml` is the executable Effigy release configuration.

## Procedure

1. Choose the next version from the workspace version and changelog. Record
   the released boundary and consumer-visible changes. Keep `Cargo.lock`
   controlled with `cargo update -w`; it is deliberately not an Effigy sync
   file because regenerating it can upgrade unrelated dependencies after
   validation.
2. Run `effigy release gates` to inspect the configured gates, then run the
   repository's release validation. `config/release.toml` requires format,
   both clippy feature shapes, workspace tests, soak, source-consumer proof,
   Rust floor proof, `effigy validate`, and `effigy qa:docs`.
3. Verify a clean source consumer builds from the tagged Git address without
   sibling checkouts. `effigy release:source-consumer` performs that proof.
   `effigy release:floor` proves the declared Rust floor. The current floor
   is Rust 1.95 from `v0.1.1`; `v0.1.0` used 1.90.
4. Use Effigy's configured release flow for preparation and execution only
   after explicit operator authorization. Do not bypass a failed gate or
   re-tag a failed release; repair and use the next patch version.
5. Consumers pin the resulting tag. For local development, use the
   gitignored Cargo patch workflow in the consumer runbook; never commit a
   path override as the released address.

Release receipts and machine-readable boundary descriptors come from the
repo-owned tasks and `signal-supervisor-tools`. The
[packaging contract](010-publication-grade-packaging-manifest-and-release-receipt-contract.md)
defines their meaning.

## Sandbox broker distribution

A normal Cargo dependency does not supply the
`signal-plugin-sandbox` executable. Consumers provide a compatible,
already-built broker through `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND`; startup
does not compile it from a checkout. Provisioning is a separate step through
`effigy broker:provision` or an equivalent prebuilt distribution. The
environment-variable arguments and working-directory controls remain part of
the boundary. Release-shipped broker assets are a later distribution choice.
