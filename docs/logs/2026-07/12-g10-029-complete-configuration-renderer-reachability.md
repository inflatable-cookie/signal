# Complete Configuration Renderer Reachability

Date: 2026-07-12
Roadmap: `g10.029`
Batch: `29.6BK` checkpoint
Status: complete; objective grid execution ready

## Result

The BJ renderer now consumes the complete frozen tuning configuration. Focused
release evidence changes one dimension at a time and proves that each reaches
its intended mechanism:

- both non-baseline geometries change complete output
- conservative sensitivity changes selected points and schedule
- unity strengths `0.0` and `0.5` change the exact-closing schedule from `1.0`
- confidence-owned and frequency-limited reset scopes change phase behavior
  from short-only
- disabling vertical alignment changes phase behavior without changing its
  schedule or magnitude input

The frequency-limited proof includes a dominant `5003 Hz` component so the
out-of-band reset path is live. The ordinary `80..2000 Hz` protection remains
part of the same branch.

## Hard-Gate Evidence

Nine focused configurations have:

- exact output length
- zero uncovered samples
- zero non-finite values
- zero boundary failures
- zero event-order failures
- nonzero linked-decision hashes
- deterministic study, schedule, magnitude, phase, and output hashes

The frozen BH union, BI study/schedule, and BJ complete phase/synthesis tests
continue to pass after parameterization.

## Boundary

This checkpoint proves configuration reachability only. It does not score the
`108` configurations, read holdout evidence, construct a Pareto frontier, or
export listening candidates.

## Next Task

Continue Batch 29.6BK. Execute all `108` wired configurations on synthetic and
the nine development rows, apply hard gates and Pareto selection, then export
at most three concealed candidates. Keep holdout closed.
