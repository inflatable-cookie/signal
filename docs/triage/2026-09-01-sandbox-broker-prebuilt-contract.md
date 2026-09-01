# Sandbox broker consumer decision

Status: decided
Date: 2026-09-01
Owner: Signal packaging and consuming-Signal documentation
Canonical tracker: `/Users/tom/Dev/projects/loophole/PAPERCUTS.md`

## Decision

Adopt option 2 for the `signal-plugin-sandbox` consumer boundary:

- Cargo dependencies do not promise a dependent package's executable on stable
  Cargo.
- A consumer must receive a compatible, already-built broker executable through
  `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND`.
- Signal's consumer runbook and its developer/CI provisioning path must make
  that boundary explicit and reproducible.
- Consumer startup must not compile Signal's broker from a Signal checkout on
  first use.

The existing environment-variable arguments and working-directory controls
remain part of the boundary. A provisioning step may build or retrieve the
broker before the consumer run, but that step is explicit and separate from
consumer startup.

## Deliberately deferred

Option 1 (release-shipped broker assets) remains a later product-distribution
decision. Option 3 (stable Cargo artifact dependencies) remains unavailable;
the current Cargo limitation is not repaired by adding an empty library
target.

The Signal implementation lane must stay within Signal documentation,
developer/CI provisioning helpers, and focused proof. It must not change
Loophole, invent a new broker protocol, or claim that a normal Cargo
dependency supplies the executable.

This note records the decision only. The worker handoff is the execution
authority.
