# g10.029 Adaptive Single-Frame Failure Attribution

Date: 2026-07-13

## Scope

Batch 29.6BT traces every failing Rule 30N tone, isolated-event, and dense-event
row through the frozen ordinary and combined renderers. No algorithm policy,
threshold, corpus, or holdout state changes.

## Result

The failure boundary is split and specific.

- `30` frozen rows; `25` retain their Rule 30N hard failure
- `2,298` per-frame phase records
- `78` event-overlapping diagonal-dual contribution records
- `14` failures begin in physical-frequency phase transport
- `10` begin in event ownership/frame attachment
- one combined-only failure begins in event correction
- zero earliest failures belong to vertical locking or diagonal-dual synthesis
- maximum rendered tone error remains `6.842e-4` radians/sample
- maximum traced frame-frequency error is `3.174e-2` radians/sample
- same-resolution maximum is `3.174e-2`; transition maximum is `7.199e-3`
- dominant-bin ownership changes `738` times
- none of `18` injected event instances is selected
- only six injected instances coincide with an exact frame centre
- isolated and dense maxima remain `496` and `896` sample frames
- row, frame, coefficient, phase, contribution, output, and repeat hashes match
- aggregate evidence hash `ddca308a7f60f39e` repeats

The sole combined-only regression is the `0.75` mid tone. Ordinary transport
passes; combined mode applies `37` event and `37` vertical assignments. Event
correction runs first and is the earliest changed phase boundary.

## Decision

Fixed-bin phase state is invalid once a previously weak or dormant bin becomes
the current spectral owner. Resolution transitions are not the sole cause;
same-resolution errors are larger. Phase state must follow active peak
trajectories and initialize new owners from current analysis phase.

Resolution points cannot own transient anchors. The frozen study detects none
of the injected isolated/dense attacks at their sample positions, so event
correction receives no exact event-centred frame. A separate linked onset path
must refine attacks to sample-frame anchors and attach them exactly to the
global map.

## Next Task

Execute Batch 29.6BU under Rule 30P. Keep the complete Rule 30N rerun, corpus,
holdout, listening, tuning, linked stereo, dynamic ratio, and routing closed.
