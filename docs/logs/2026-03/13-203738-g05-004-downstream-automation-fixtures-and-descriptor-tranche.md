# 2026-03-13 20:37:38 GMT - g05.004 downstream automation fixtures and descriptor tranche

## Summary

Completed `g05.004` Batch 4.2 by materializing the first broader shared
automation fixtures and machine-readable downstream automation descriptor on
top of the mandatory-versus-optional contract split.

## Work completed

- added `signal-supervisor-tools --describe-downstream-automation`
- added `effigy acceptance:downstream-release --repo .`
- added `effigy acceptance:downstream-depth --repo .`
- added `effigy acceptance:downstream-automation --repo .`
- kept the optional depth path typed by using `signal.supervisor.export` JSON
  scenario runs rather than log-only soak output
- moved `g05.004` forward to Batch 4.3

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_downstream_automation_mode`
- `cargo test -p signal-supervisor-tools downstream_automation_json_reports_mandatory_and_optional_fixtures`
- `cargo run -p signal-supervisor-tools -- --describe-downstream-automation --format=json`
- `effigy acceptance:downstream-automation --repo .`
- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Residual risk

The wider automation seam is now explicit and runnable, but it still lacks a
credible fail-gate policy. Mandatory release acceptance and optional depth are
separated, yet the promotion criteria between them still belong to Batch 4.3.

## Next task

Continue `g05.004` with Batch 4.3 by defining the first credible fail-gate
policy for the widened downstream automation surface, keeping expensive or
optional depth explicit when it stays out of the fast release path.
