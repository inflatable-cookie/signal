# g10.031 Linked STN Protocol Binding

Date: 2026-07-21
Status: Batch 31.57 complete; Batch 31.58 ready

## Outcome

The canonical linked-STN brief now freezes fresh identity
`ConformanceBoundLinkedStnNoiseMorph` under Contract `085` Rule 11. Work starts
from Batch 31.57 closeout commit based on `efbd56242ab12364cc83025e4f1b8360c140d1de`
and uses exact worktree, branch, module, test prefixes, conformance ledger,
receipt root, and local acoustic ref.

Compile, construction `1/1`, and structural `18/18` may iterate against frozen
authority. One clean tree must pass that sequence twice before the immutable
acoustic ref exists. Acoustic owners compile but cannot execute or emit
inspectable audio before the ref. Synthetic, concealed mono, speaker, and
independent stereo stages then run once in order from that identity.

## Authority Audit

The brief now names the complete structural corpus, exact synthetic WAV byte
identities, helper estimators, long-form source hashes and mono downmix,
linked-stereo balance calculation, cleanup behavior, and pass disposition.

Two inherited descriptions were corrected from retained artifacts:

- synthetic support uses the retained half-cosine entry and exit fades, not
  linear fades
- `Y07` compares mapped-gap RMS with the complete mapped active support, not
  two adjacent regions

The retained files and published comparator values already use those rules.
No source bytes, DSP formula, seed, reference number, threshold, assertion,
comparator configuration, or listening policy changed.

## Scope

Documentation only. No DSP, candidate source, test, harness, dependency,
production route, cache, product API, Loophole, or Chorus surface changed. The
three pre-existing plugin worktree edits remain outside this batch.

## Validation

- `git diff --check`
- `effigy qa:docs`
- `effigy qa:northstar`
- `effigy health`
- `effigy validate`

All passed. The earlier `effigy doctor` output remained limited to the known
god-file and attention-marker findings.

## Next Task

Run Batch 31.58 in isolated worktree `signal-candidate-31-58` on branch
`candidate/g10-031-conformance-bound-linked-stn-noise-morph`. Start fresh from
the exact Batch 31.57 closeout commit, implement only the frozen private
renderer, and finish conformance before creating the acoustic ref. Do not
recover deleted candidates, alter `main`, merge, or push.
