# Backlog: Post-g05 Publication Promotion And Shared Acceptance Depth

Status: promoted
Priority: medium
Estimated effort: multi-batch generation
Source: g05.005

## Problem

`g05` closed the widened shared Signal boundary, but several credible next
steps remain deliberately outside the finished release gate: publication and
distribution automation beyond the current repo-owned manifest descriptor,
promotion of broader shared acceptance depth into stronger release evidence,
and stabilization of the currently deferred server-host soak lane.

## Proposed approach

Open the next generation only when maintainers want to promote those deferred
scopes without reopening host-local ownership or consumer-specific policy.
Keep the next queue inside Signal-owned reusable boundaries:

- deepen publication/distribution receipts only where Signal can stay
  authoritative instead of inheriting app-local release wrappers
- harden the server-host soak and related fault/recovery acceptance paths until
  they can participate in stronger shared gating
- decide which currently advisory shared acceptance lanes should become
  mandatory release evidence and which should remain bounded optional depth
- keep any broader automation outputs typed, machine-readable, and repo-owned

## Promotion trigger

Promote this backlog item when at least one of the following becomes true:

- maintainers want stronger publication/distribution automation than the
  current host-free manifest and boundary descriptors
- the deferred `server soak` lane becomes important enough to stabilize and
  promote toward a stronger shared release gate
- broader analysis or longer-running shared confidence work needs to move from
  advisory depth toward explicit release evidence

## Success criteria

- [ ] the next generation names which deferred `g05` scopes are now promoted
- [ ] the new queue keeps release and acceptance ownership inside Signal-owned
  contracts, descriptors, receipts, and Effigy tasks
- [ ] broader publication or soak depth does not reintroduce consumer-local
  orchestration or host-local reconstruction

## Risks

- publication work can sprawl into installer, registry, or distribution
  workflow detail that does not belong in Signal
- stronger acceptance gating can become expensive integration sprawl instead of
  shared contract proof
- server-host soak promotion can blur runtime hardening work with app-specific
  deployment concerns if the queue is not kept host-neutral

## Next Task

PROMOTED into `g06` on 2026-03-13, but only in the narrowed form that still
belongs inside Signal-owned reusable boundaries: stronger runtime acceptance,
server-host soak promotion candidates, and deeper machine-readable runtime
evidence. Publication/distribution work that does not directly move reusable
runtime value forward remains deferred until maintainers explicitly reopen it.
