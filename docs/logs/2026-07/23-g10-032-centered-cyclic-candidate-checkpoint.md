# g10.032 Centered Cyclic Candidate Checkpoint

Date: 2026-07-23
Status: Batch 32.5 complete; Batch 32.6 ready

## Result

Implemented the frozen `CenteredCompressedAnchorCyclic` renderer once in the
isolated worktree. The private candidate uses:

- one exact centred rational source/output map
- one fixed `5..90 ms` manual cycle, `48 ms` neutral
- two neighbouring forward unit-rate reads per output
- complementary raised-cosine crossfade
- one linked mono/stereo schedule
- direct exact crop, exterior zero, no implicit fade
- duration-independent `256 KiB` maximum working state
- deterministic scalar `O(C*T)` execution

No candidate DSP, harness, fixture, API, dependency, route, cache, Loophole, or
Chorus change entered `main`.

## Checkpoint

- branch: `candidate/g10-032-centered-compressed-anchor-cyclic`
- commit: `4600d228286797d22e4f4d5ca4efa997835fc4b2`
- tree: `fa1fc8031a4aab4302b778474702e658784d8a64`
- ref:
  `refs/signal-evidence/creative/centered-compressed-anchor-cyclic/32-5-acoustic`
- build: release, one test thread, no retry, `600 s` owner deadline
- comparator: REAPER `7.69/macOS-arm64`, mode `983040`, `44.1 kHz`, `24-bit`
- comparator rows: `15` musical, `30` synthetic mono, `18` synthetic stereo
- comparator manifest:
  `eb5384681767dfd36e8daf81809a95d51a79f6cb178f0705fe4cffce9ecccacd`
- historical comparator group:
  `5bb7b55456065d8f3d69c7229abc117eacb9280cf298a779b634598a19663e11`

## Frozen Specs

- render: ratios `2/4/8`; cycles `5/48/90 ms`; rates
  `8000..192000 Hz`; maximum ratio `8`; peak tolerance `2e-6`
- evidence: structural `339/168`; synthetic `183/201`; source
  `44100 Hz`, `88200` frames; active `[22050,66150)`; ramp `2048`
- comparator: `63` rows; mode field `0.0025`; output fade `10 ms`
- memory: `262144` bytes; maximum cycle `17280` frames; zero processing
  allocations
- run: two conformance rounds; release; serial; no retry

## Conformance

Both clean rounds passed:

- release compile: `2/2`
- construction: `1/1` twice
- structural: `9/9` twice
- per-owner receipts: byte-identical across rounds

Receipt SHA-256:

- `S01` `173661ac17189418b080aef200f4a822877f6d5a7e597c64cf8fe8b27b8d6585`
- `S02` `32b5610ed32fbfd60984dc33de1b35d719338b9a5bc6cb4dfc5c735e1dec26e4`
- `S03` `5b113b837707fa71befdde262e918cfc4aaf729afa496957137ada436338cd59`
- `S04` `fc1a468ba26d03116b127629155ce0c9a871a91174a28ba683e1f490aae037ff`
- `S05` `912dccacaf83b9ab13f1bc47931c945c0c307cb5c7bbf09e4145805943e30ffe`
- `S06` `5eebc1a2d8c698fcba352f4977c2c8d33686e8ed23490354868aa9b0290ad2ef`
- `S07` `ef8e9aac57db4122a75654336b36f4c70324c1e8058ad9d5daf1de7b4d3c9cc5`
- `S08` `c5c1b442f09147f5d890114b1d7a260b51ff5de22b13de1496a5144d3ee4b5e9`
- `S09` `ba0093464eb396493436aa8457f33c23502e50a5a9133cbf3c727be1e0d6e277`

`Y01..Y06` did not run. Only bounded structural test renders exist; no
synthetic-gate, long-form, or listening output was generated.

## File Identity

- `.config/nextest.toml`
  `67ba16f998d8e46351b03f90211156e45d65af214b91779182d7dda4a42fb413`
- `candidate-evidence/g10-032/32-5/conformance.tsv`
  `7365e35f0922277eaa240bf8e497d5098bcdd8c56c0a22fd714c0022c2afff3a`
- `evidence.rs`
  `d5bbcdfc3c7460350ec8bd209726ea7c2d6b1d7779974abe08d147519f4af7fa`
- `interpolate.rs`
  `c3aaf8d752ae4f41f0505219efd47bfead9f21738c04e874d995660a7ebbb310`
- `mod.rs`
  `9c52849b5a77ffb14dcfef253c0affadb70cf7a227bf6fe386a41b5613d93848`
- `plan.rs`
  `8edbbc7aefd838e04df43d069a03136f7be2f00e87637268b22fa6363a9a00a6`
- `schedule.rs`
  `062ff5415b0cc51c77b4b593a645ef2062aad9731692e27e17ecbf5aa79a2f87`
- `synthesis.rs`
  `a08310bc1dd7210d64d3404979f3d8d3207837328f4eafdd7d01399d32cb56e5`
- `tests.rs`
  `637f1119ee659e1bee18eee6cab768b8aaef2eda857f6ca1eeb412079d05a18a`
- `crates/signal-dsp-stretch/src/lib.rs`
  `ef978f473144d39e39c0008cfc1a78db9555eda0a01925daf8b62befde7d8d56`
- `Cargo.lock`
  `e3848a40d2ea1ff88a0e036df40d1fefa56c7aca950a95262c1d8c5668fd394d`

Toolchain:

- `rustc 1.96.0 (ac68faa20 2026-05-25)`
- `effigy v0.8.17+local.2838eb4`
- `cargo-nextest 0.9.133 (65e806bd5 2026-04-14)`
- macOS `26.5.2` build `25F84`, `arm64`

## Risk

- all synthetic acoustic gates remain unrun
- direct linear reads may lose high-frequency polish
- short cycles may buzz; long cycles may pump or echo
- comparator-relative hard controls may stop the candidate before listening
- mono musical sufficiency and independent linked-stereo review remain unknown

## Next Task

Execute Batch 32.6 only from the immutable ref. Run `Y01..Y06` individually in
numeric order and stop on the first hard failure. Run exact `16x` typed
rejection only if all six pass, then follow the frozen mono, cycle-direction,
stereo-objective, speaker, and independent-listener order without changing
candidate code or evidence authority.
