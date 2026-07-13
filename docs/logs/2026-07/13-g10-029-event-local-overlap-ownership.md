# Event-Local Overlap Ownership

Date: 2026-07-13
Roadmap: `g10.029`
Batch: `29.6BX`
Contract: `082`, Rule 30S

## Result

The dense replica has one frame owner. The non-anchor `512`-frame analysis at
source `8192`, projected output `16385`, contributes the complete `0.787177`
peak at output `16382`. The exact attack frames at sources `8064` and `8320`
alone produce the real target amplitudes `1.0` and `0.75`.

The successor now treats a frame as a conflicted bridge only when it straddles
multiple accepted anchors and their projected owner supports no longer overlap.
Within `64` source frames of each anchor, that bridge receives linearly
interpolated boundary background. Anchor frames retain the original attack.

## Bounded Proof

- pre dense errors: `[[0,0],[0,0],[0,262]]`
- post dense errors: `[[0,0],[0,0],[0,0]]`
- `0.75x` and `1.5x`: bit-identical outputs
- `2.0x` changed source samples: `2`
- maximum real-target amplitude delta: `0`
- replica amplitude: `0.787177` to `0`
- contribution hashes: `b5fa80b289fcf1b4` to `3a77bac045f1d468`
- evidence hash: `adf37bdd72012e19`

The complete unchanged Rule 30Q matrix then passes all `48` control/ratio rows
with zero hard failures and zero regressions. Evidence hash
`dec15b718aa27de9` repeats.

## Boundary

This is release-test successor work. It changes no current product renderer,
cache identity, linked-stereo path, dynamic-ratio path, realtime callback, or
routing surface. No corpus, holdout, comparator implementation, or listening
audio was read.

## Next

Run Rule 30T on the frozen nine-row mono development set. Compare the selected
successor with current Signal and captured external evidence. Keep holdout,
listening, tuning, stereo, dynamic ratio, cache, and routing closed.
