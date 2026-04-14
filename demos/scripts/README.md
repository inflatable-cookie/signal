# Demo Runners

Status: active
Updated: 2026-04-14

## Purpose

This folder holds the Bun/TS runner layer for the Signal demo registry.

It exists to turn manifests plus repo-owned commands into stable receipts and
rendered operator views.

## Shape

- `lib/demo-runtime.ts`
  - shared command execution, receipt writing, target-dir control, and bounded
    process helpers
- `lib/operator-view.ts`
  - shared rendered companion HTML builder for the headless operator-view
    family
- `run_<surface>_demo.ts`
  - one runner per live demo surface

## Working Rule

- keep runner names aligned with the manifest surface names
- prefer shared helpers over copying subprocess or rendering logic between
  runners
- only add special-purpose orchestration when the demo genuinely needs it
  beyond the shared runtime layer
- interactive demos may justify custom serving and launch logic, but that
  should stay isolated to that demo rather than leaking back into the headless
  family

## Current Boundary

- headless operator-view and platform-boundary demos share the same Bun/TS
  runtime layer
- the plugin capability browser is the only materially custom runner

## Next Task

Keep shrinking custom logic into `lib/` where it is genuinely reusable, but do
not force the browser runner into the headless shape if that would hide real
interactive complexity.
