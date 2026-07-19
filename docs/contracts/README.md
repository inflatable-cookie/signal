# Contracts

Status: active
Updated: 2026-07-19

## Why this section matters now

Contracts freeze the reusable boundaries that Signal consumers should be able to
rely on.

## Scope

Use this section for:

- stable reusable-DSP and runtime boundary contracts
- export/report contracts
- host-edge and policy contracts when prose architecture is not precise enough

## Current Baseline

- `001-working-rules.md` for repository execution posture
- `001-shared-dsp-and-host-boundary.md`
- `002-supervisor-export-schema-and-report-boundary.md`
- `003-crate-maturity-and-public-runtime-boundary-baseline.md`
- `004-runtime-multicore-scheduling-and-anticipative-execution-contract.md`
- `005-runtime-work-orchestration-and-deferred-service-policy.md`
- `006-runtime-hardware-portability-and-clock-domain-contract.md`
- `007-plugin-backend-and-host-neutral-delegation-contract.md`
- `008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`
- `009-shared-host-convenience-api-and-consumer-edge-contract.md`
- `010-publication-grade-packaging-manifest-and-release-receipt-contract.md`
- `011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`
- `012-runtime-interruption-taxonomy-and-resumability-contract.md`
- `013-recording-continuity-midi-capture-and-checkpoint-contract.md`
- `014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`
- `015-offline-render-recovery-and-resumability-contract.md`
- `016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`
- `017-per-block-execution-timing-and-pressure-snapshot-contract.md`
- `018-graph-critical-path-hot-node-and-worker-lane-instrumentation-contract.md`
- `019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md`
- `020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md`
- `021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md`
- `022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`
- `023-generic-midi-note-expression-and-plugin-event-model-contract.md`
- `024-plugin-preset-state-interchange-portable-recall-and-ara-context-contract.md`
- `025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`
- `026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md`
- `027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md`
- `028-media-indexing-waveform-analysis-and-preview-service-contract.md`
- `029-analysis-metadata-extraction-and-library-service-contract.md`
- `030-fault-injection-harness-and-multi-backend-acceptance-contract.md`
- `031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md`
- `032-canonical-multichannel-layout-and-channel-role-contract.md`
- `033-sidechain-routing-and-secondary-input-execution-contract.md`
- `034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`
- `035-plugin-complex-io-topology-and-multi-output-instrument-contract.md`
- `036-spatial-adapter-execution-contract.md`
- `037-surround-bed-object-and-mix-policy-expansion-contract.md`
- `038-lv2-adapter-baseline-and-linux-native-plugin-lifecycle-contract.md`
- `039-linux-cross-adapter-plugin-parity-and-sandbox-policy-contract.md`
- `040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md`
- `041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md`
- `042-external-midi-endpoint-graph-and-device-identity-contract.md`
- `043-midi-2-0-mpe-and-richer-controller-expression-contract.md`
- `044-control-surface-transport-mapping-and-feedback-contract.md`
- `045-advanced-hardware-extensibility-and-scripting-safe-device-policy-contract.md`
- `046-sample-domain-time-stretch-engine-contract.md`
- `047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md`
- `048-post-warp-render-cache-and-transform-artifact-contract.md`
- `049-low-latency-audition-scrub-and-preview-transform-service-contract.md`
- `050-multichannel-linux-time-stretch-and-control-surface-acceptance-contract.md`
- `051-generation-closeout-and-loophole-feature-readiness-gate-contract.md`
- `052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`
- `053-jack-transport-graph-and-backend-native-coordination-contract.md`
- `054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md`
- `055-lv2-worker-urid-patch-and-extension-negotiation-contract.md`
- `056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md`
- `057-immersive-object-rendering-and-room-policy-substrate-contract.md`
- `058-speaker-deployment-fold-down-and-monitoring-scene-contract.md`
- `059-renderer-capability-negotiation-and-immersive-export-contract.md`
- `060-advanced-control-surface-display-motor-and-haptic-transport-contract.md`
- `061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md`
- `062-preview-output-routing-audition-sink-and-low-latency-device-policy-contract.md`
- `063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md`
- `064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md`
- `065-live-external-midi-device-ownership-and-backend-parity-contract.md`
- `066-cross-backend-device-protocol-and-live-workflow-acceptance-contract.md`
- `067-live-linux-backend-acceptance-and-failure-injection-contract.md`
- `068-immersive-render-and-monitoring-acceptance-contract.md`
- `069-control-surface-and-preview-workflow-acceptance-contract.md`
- `070-integrated-live-ownership-and-workflow-acceptance-contract.md`
- `071-generation-closeout-and-downstream-workflow-readiness-gate-contract.md`
- `072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`
- `073-native-backend-device-truth-and-coreaudio-implementation-contract.md`
- `074-shared-host-runtime-execution-and-recovery-unification-contract.md`
- `075-runtime-public-interface-decomposition-and-internal-assembly-boundary-contract.md`
- `076-low-level-correctness-safety-and-protocol-hardening-contract.md`
- `077-dsp-fidelity-semantic-calibration-and-analysis-realism-contract.md`
- `078-rhythm-continuity-failure-containment-and-policy-normalization-contract.md`
- `079-interactive-demo-binary-and-crate-capability-proof-contract.md`
- `080-production-readiness-grade-and-generation-release-gate-contract.md`
- `083-vst3-discovery-diagnostic-outcome-contract.md`
- `084-stretch-candidate-isolation-and-promotion-contract.md`
- `085-creative-time-stretch-product-and-routing-contract.md`

## Rule

Add a new contract only when the boundary needs stronger guarantees than
`architecture/` alone can provide.

## Next Task

Use `docs/contracts/contract-index.md` and `001-working-rules.md` as the
contract front doors. Contract `084` and roadmap `g10.030` keep the transparent
successor program closed. Contract `085` governs the separate creative path;
run `g10.031` Batch 31.7 linked-relation architecture reassessment only. No
strict spec lane is open.
