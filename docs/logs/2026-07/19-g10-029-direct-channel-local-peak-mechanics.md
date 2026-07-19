# Direct Channel-Local Peak Mechanics

Date: 2026-07-19
Batch: 29.7BA
Status: complete

## Changed

The direct candidate now builds peak, valley, fallback, and predecessor maps
per channel and per scale. Locked channel-atoms retain their requesting peak,
select a possible trajectory channel at the requesting atom, require equal
supported predecessor identity, and re-anchor the selected ordinary advance to
that predecessor. Magnitude and current requesting offset stay local.

State reports now count borrowed and local locked channel-atoms, committed
trajectory-channel switches, and channel-peak disagreements. Storage remains
`2CP` region records, `2CP` phase values, and `P` terminal states.

## Evidence

The frozen joint baseline collapses the staggered fixture onto peak `11`.
Corrected records retain channel peaks `9` and `11`, borrow without replacing
the requesting peak, and preserve magnitude and within-region offset at
`1e-12`. The fixture repeats at `fcbdfd991bd04db1`.

Incompatible and unsupported predecessors, unsupported current owner, lower-
bin/channel ties, exact `6000 Hz`, swap, silence/recovery, every terminal state,
all proof rates, fixed storage, pre-mutation rejection, finiteness, and repeat
pass. Terminal, relation, and representation receipts are
`5ae654162d4ed279`, `2b8104525bad0418`, and `fdf90f6127749341`.

No objective or corpus audio ran.

## Next Task

Run Batch 29.7BB. Freeze one failure-first objective candidate from these
receipts and the unchanged AX evidence order before any renderer or corpus
execution.
