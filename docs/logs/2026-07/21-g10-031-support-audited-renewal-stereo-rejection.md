# g10.031 Support-Audited Renewal Stereo Rejection

Date: 2026-07-21
Batch: 31.36
Status: complete; candidate rejected at source-relative stereo admission

## Result

Fresh checkpoint `5d8eaf4555a783ae8efee6479c972c8acca1aed4` passed compile,
construction `1/1`, structural `15/15`, and synthetic `9/9` without
post-checkpoint repair or rerun. `Y02` completed its listening-led pitch
matrix. `Y08` passed from the sole frozen `SYNTHETIC_SUPPORTS` authority.

Concealed mono passed as `15/15` ties against PaulXStretch. The operator found
all rows broadly similar and usable. Signal carried minor extra low-frequency
noise, entered more gently, and ended more abruptly. PaulX began more solidly
and carried a longer fade-out. Assembly added no fade.

## Stereo Stop

An initial stereo assembly duplicated the mono sources. Its zero balance
errors were vacuous and were discarded before review. The corrected one-shot
gate used the exact retained stereo originals and same-source PaulX captures.
It rendered five sources at `4x`, `8x`, and `16x`, each at `space=0`, `0.5`,
and `1`: `45` candidate rows.

All three `16x` bass rows exceeded the `1.50 dB` mapped-window balance limit:
`1.998162..2.000356 dB`. All three `16x` full-mix rows reached
`9.366481..9.418990 dB` and reversed local channel dominance. Whole-render and
three-band errors on those rows remained below `0.027 dB`. The dominant cause
is local linked-image instability across the source/output map, not global
channel gain or spectral balance.

The objective miss rejects the complete candidate. Speaker pre-screen and
eligible independent stereo listening did not open. The latter was not
waived.

## Cleanup And Authority

Deleted the disposable worktree, branch, checkpoint reference, private
six-file module, tests, build state, and listening assembly. The deleted
checkpoint is no longer branch-reachable; recovery would require Git object
salvage before pruning. No DSP, test, harness, fixture, API, cache, route,
Loophole, or Chorus surface entered `main`. Nothing was pushed.

Batch 31.25 and Batch 31.36 are now two complete renewal candidates with
terminal linked-stereo failures: one global source-balance inversion, one
local mapped-window dominance reversal. Contract `084` requires architectural
reassessment rather than another local relation-law variant.

Repo posture remains `baseline-routing`; no strict lane is active. Batch 31.37
is ready as docs-only renewal stereo-ownership reassessment.

## Validation

- `git diff --check`
- `effigy qa:docs`
- `effigy qa:northstar`
- `effigy health`
- `effigy validate`

All passed. `effigy doctor` retained the pre-existing god-file error and
attention-marker warning; this batch did not expand scope to address them.

## Next Task

Run Batch 31.37 only. Reconcile both complete stereo failures and inspect
retained complete-source evidence. Either identify one materially different,
source-backed complete linked-stereo owner or close renewal without closing
the PaulX-like product target. Do not implement, tune, recover candidate
source, change gates, or push.
