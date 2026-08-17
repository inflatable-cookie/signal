# Product Guardrails

Status: active
Owner: core-product
Updated: 2026-08-17
Vision refs: `docs/vision/001-signal-vision.md`

## Purpose

Keep Signal's active runtime and host work inside explicit product and
engineering guardrails so roadmap execution does not drift into fake depth,
host-local duplication, or unsafe shortcuts.

## Guardrails

- prefer real runtime and adapter behavior over scaffolded or synthetic truth
  when a milestone claims concrete plugin, backend, or execution depth
- do not leave placeholder discovery, broker, device, or recovery behavior in
  place once a lane is supposed to prove the real path
- preserve realtime safety on audio-thread and timing-sensitive paths; if a
  change needs blocking or allocation-heavy work, keep it off the realtime path
- prefer shared runtime or shared-host-support policy over parallel local and
  server implementations when the semantics are genuinely the same
- keep environment-specific differences explicit at the edge rather than
  flattening them into vague shared helpers
- prefer explicit degraded behavior and typed failure receipts over silent
  fallback or hidden recovery
- do not widen active lanes into UI, workflow, or downstream-app behavior that
  Signal does not own
- avoid compatibility shims unless the roadmap or contract explicitly asks for
  them
- do not claim capability through roadmap prose, demos, or docs alone; the
  implementation or a clearly deferred note must back the claim

## Stop Conditions

- a batch would keep synthetic device or plugin truth where the lane is meant
  to prove a real path
- host duplication is being preserved for convenience rather than because the
  environment boundary is genuinely different
- the change would push unsafe or unbounded work into realtime-sensitive paths

## Next Task

Use these guardrails with the active generation front door and
`docs/roadmaps/strategic-runway.md` so new batch cards stay focused on real
runtime and host behavior rather than scaffolded or downstream-app scope.
