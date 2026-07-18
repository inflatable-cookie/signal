# g10.029 Relation-Owned Sliced Frame Stage A

Date: 2026-07-18
Batch: 29.7AA Stage A
Status: complete

## Scope

Prove the fixed sliced representation before relation-owned material transport.
Use the frozen `16384/8192/512` geometry, `4096/2048/1024` atom supports, and
`750 Hz`/`6 kHz` ownership boundaries. Keep material, listening, and product
work closed on any miss.

## Evidence

Identical sine analysis and synthesis windows form a two-slice square
partition. The existing painless inner dual reconstructs reflected whole-render
boundaries across lengths `[1, 4095, 8192, 12289, 220500]`. Peak error is
`4.44e-16`; conjugate closure is exact after explicit real-spectrum pair
construction. Crop, coverage, silence, hard pan, swap, polarity, scaled
duplicate, boundary, repeat, and finite-value gates have zero failures.

The five identity lengths require `[2, 2, 2, 3, 28]` slices. Boundedness lengths
`[8192, 65536, 220500]` retain at most two live slices and `86016` peak live
coefficients. Counted work is exactly `1111425` units per slice and therefore
linear in slice count. Evidence hash: `0830ec12fa0bcde7`.

## Decision

Stage A passes. Stage B may run once with the frozen relation-owned material
operator. Listening, production routing, dynamic ratio, realtime, and Batch
29.8 remain closed until the complete objective candidate passes.

## Next Task

Run Batch 29.7AA Stage B once. Add the frozen relation law and 29.7Y material
operator to the passing sliced frame. Run synthetic and exact mechanics first,
then the `48`-row calibrated stereo gate. Stop before the long mono corpus on
any miss.
