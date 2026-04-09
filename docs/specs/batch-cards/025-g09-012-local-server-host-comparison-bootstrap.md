# 025 - g09.012 Local Server Host Comparison Bootstrap

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.012
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Auto-start next card: no

## Objective

Take the next honest `g09.012` seam by turning the newly bootstrapable
`signal-host-local` and `signal-host-server` binaries into one bounded
comparison demo surface with a manifest, launch task, scenario notes, and
machine-readable receipt.

## Reactivation Note

The blocking CLAP host sandbox gap is now closed. This card is active again
because the local and server host binaries boot on their real CLAP path, so the
comparison wrapper no longer sits on top of a known deferred capability gap.

## Scope

- stay inside the existing host binaries and their current output surfaces
- build a repo-owned comparison wrapper around those binaries without changing
  their runtime behavior beyond what `024` already fixed
- capture shared recovery and execution posture that both host binaries already
  export
- keep unsupported and deferred areas explicit rather than hiding them
- do not widen into plugin capability browsing, hardware live demos, or a full
  operator shell

## Steps

1. Freeze the bounded host comparison seam from `g09.012` and contract `079`.
2. Add a host comparison manifest, operator notes, receipt path, and Effigy
   launch task under `demos/`.
3. Implement a repo-owned comparison script that runs both host binaries,
   captures their existing summary lines, and emits one machine-readable
   receipt.
4. Keep degraded or still-deferred areas explicit in the manifest and receipt
   instead of claiming unsupported capability.
5. Update the roadmap, coverage matrix, and strict currentness surfaces to
   reflect the new live host comparison surface if the batch closes cleanly.

## Acceptance Criteria

- one repo-owned launch surface runs both `signal-host-local` and
  `signal-host-server`
- the emitted receipt captures shared host comparison truth from existing host
  output, including sandbox attach, processed blocks, and readiness/running
  posture
- the manifest and receipt keep deferred plugin browsing and hardware demo
  scope explicit
- the batch stays bounded to comparison bootstrap and does not widen into a new
  host UI or plugin browser
- focused validation passes

## Evidence Required

- batch log for the next `g09.012` tranche
- validation actually run
- explicit note that plugin capability browsing and hardware demos remain
  deferred if they still do

## Stop Conditions

- the work starts redesigning host output surfaces instead of wrapping the
  current binaries
- the comparison needs fresh planning about what “shared truth” means beyond
  existing host receipts
- the batch starts building plugin browsing or hardware demo behavior

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is plugin capability browsing, hardware diagnostics, or a
continued planning pause.
