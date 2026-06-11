use super::*;
impl RuntimePreviewTransformServiceSnapshot {
    pub(super) fn derive_preview_device_policy(
        media_service: &RuntimeMediaServiceSnapshot,
        active_audition_clip_count: usize,
    ) -> RuntimePreviewDevicePolicySummary {
        let audition_active = active_audition_clip_count > 0
            || media_service.preview_state == RuntimeMediaPreviewState::Previewing;

        if !audition_active {
            return RuntimePreviewDevicePolicySummary {
                routing_posture: RuntimePreviewOutputRoutingPosture::NoPreviewOutputRouting,
                audition_sink_class: RuntimeAuditionSinkClass::NoAuditionSink,
                audition_sink_authority: RuntimeAuditionSinkAuthority::RuntimeDefault,
                low_latency_device_policy_class:
                    RuntimeLowLatencyDevicePolicyClass::NoLowLatencyDevicePolicy,
                low_latency_device_policy_outcome:
                    RuntimeLowLatencyDevicePolicyOutcome::ObserveOnlyPreview,
            };
        }

        RuntimePreviewDevicePolicySummary {
            routing_posture: RuntimePreviewOutputRoutingPosture::GuardedPreviewOutputRouting,
            audition_sink_class: RuntimeAuditionSinkClass::GuardedPreviewSink,
            audition_sink_authority: RuntimeAuditionSinkAuthority::RuntimeDefault,
            low_latency_device_policy_class:
                RuntimeLowLatencyDevicePolicyClass::GuardedLowLatencyDevicePolicy,
            low_latency_device_policy_outcome:
                RuntimeLowLatencyDevicePolicyOutcome::ObserveOnlyPreview,
        }
    }

    pub(super) fn derive_preview_workflow(
        media_service: &RuntimeMediaServiceSnapshot,
        active_audition_clip_count: usize,
        ready_clip_count: usize,
        pending_clip_count: usize,
        fallback_clip_count: usize,
        artifact_backed_clip_count: usize,
        unsupported_clip_count: usize,
    ) -> RuntimePreviewWorkflowSummary {
        let queued_preview_request_count = usize::from(media_service.previewing_asset_id.is_some());
        let previewable_asset_count = media_service.previewable_asset_count;

        let (queue_posture, queue_class, queue_outcome) =
            if queued_preview_request_count > 0 && previewable_asset_count > 0 {
                (
                    RuntimePreviewBrowserQueuePosture::SingleActivePreviewQueue,
                    RuntimePreviewBrowserQueueClass::SingleAssetAuditionQueue,
                    RuntimePreviewBrowserQueueOutcome::PreserveActivePreviewRequest,
                )
            } else if media_service.preview_state == RuntimeMediaPreviewState::Invalidated {
                (
                    RuntimePreviewBrowserQueuePosture::GuardedPreviewQueue,
                    RuntimePreviewBrowserQueueClass::GuardedPreviewRequestQueue,
                    RuntimePreviewBrowserQueueOutcome::CollapseToSingleActivePreview,
                )
            } else if previewable_asset_count > 0 {
                (
                    RuntimePreviewBrowserQueuePosture::GuardedPreviewQueue,
                    RuntimePreviewBrowserQueueClass::PreviewAssetSelectionQueue,
                    RuntimePreviewBrowserQueueOutcome::CollapseToSingleActivePreview,
                )
            } else if unsupported_clip_count > 0 {
                (
                    RuntimePreviewBrowserQueuePosture::UnavailablePreviewQueue,
                    RuntimePreviewBrowserQueueClass::UnavailablePreviewQueue,
                    RuntimePreviewBrowserQueueOutcome::TerminalPreviewQueueFailure,
                )
            } else {
                (
                    RuntimePreviewBrowserQueuePosture::NoRuntimePreviewQueue,
                    RuntimePreviewBrowserQueueClass::NoPreviewQueue,
                    RuntimePreviewBrowserQueueOutcome::IdlePreviewQueue,
                )
            };

        let (audition_posture, audition_authority, audition_continuity_outcome) =
            if active_audition_clip_count > 0
                || media_service.preview_state == RuntimeMediaPreviewState::Previewing
            {
                (
                    RuntimeMediaAuditionOrchestrationPosture::DirectRuntimeAuditionOrchestration,
                    RuntimeMediaAuditionOrchestrationAuthority::RuntimeDefault,
                    RuntimeMediaAuditionContinuityOutcome::PreserveActiveAudition,
                )
            } else if media_service.preview_state == RuntimeMediaPreviewState::Invalidated {
                (
                    RuntimeMediaAuditionOrchestrationPosture::GuardedRuntimeAuditionOrchestration,
                    RuntimeMediaAuditionOrchestrationAuthority::GuardedRuntimeOverride,
                    RuntimeMediaAuditionContinuityOutcome::CollapseToGuardedAudition,
                )
            } else if previewable_asset_count > 0 {
                (
                    RuntimeMediaAuditionOrchestrationPosture::GuardedRuntimeAuditionOrchestration,
                    RuntimeMediaAuditionOrchestrationAuthority::RuntimeDefault,
                    RuntimeMediaAuditionContinuityOutcome::ResumePreviewAudition,
                )
            } else if unsupported_clip_count > 0 {
                (
                    RuntimeMediaAuditionOrchestrationPosture::UnavailableAuditionOrchestration,
                    RuntimeMediaAuditionOrchestrationAuthority::GuardedRuntimeOverride,
                    RuntimeMediaAuditionContinuityOutcome::TerminalAuditionFailure,
                )
            } else {
                (
                    RuntimeMediaAuditionOrchestrationPosture::NoAuditionOrchestration,
                    RuntimeMediaAuditionOrchestrationAuthority::RuntimeDefault,
                    RuntimeMediaAuditionContinuityOutcome::IdleAuditionContinuity,
                )
            };

        let (
            transform_scheduling_posture,
            transform_scheduling_authority,
            transform_scheduling_outcome,
        ) = if artifact_backed_clip_count > 0 {
            (
                RuntimePreviewTransformSchedulingPosture::DirectRuntimeTransformScheduling,
                RuntimePreviewTransformSchedulingAuthority::PreviewDemandDerived,
                RuntimePreviewTransformSchedulingOutcome::PreferArtifactBackedPreview,
            )
        } else if ready_clip_count > 0 {
            (
                RuntimePreviewTransformSchedulingPosture::DirectRuntimeTransformScheduling,
                RuntimePreviewTransformSchedulingAuthority::PreviewDemandDerived,
                RuntimePreviewTransformSchedulingOutcome::PreserveReadyTransformSchedule,
            )
        } else if pending_clip_count > 0 || fallback_clip_count > 0 {
            (
                RuntimePreviewTransformSchedulingPosture::GuardedRuntimeTransformScheduling,
                RuntimePreviewTransformSchedulingAuthority::GuardedRuntimeOverride,
                RuntimePreviewTransformSchedulingOutcome::CollapseToFallbackTransforms,
            )
        } else if unsupported_clip_count > 0 {
            (
                RuntimePreviewTransformSchedulingPosture::UnavailableTransformScheduling,
                RuntimePreviewTransformSchedulingAuthority::GuardedRuntimeOverride,
                RuntimePreviewTransformSchedulingOutcome::TerminalTransformSchedulingFailure,
            )
        } else {
            (
                RuntimePreviewTransformSchedulingPosture::NoTransformScheduling,
                RuntimePreviewTransformSchedulingAuthority::RuntimeDefault,
                RuntimePreviewTransformSchedulingOutcome::IdleTransformScheduling,
            )
        };

        RuntimePreviewWorkflowSummary {
            queue_posture,
            queue_class,
            queue_outcome,
            audition_posture,
            audition_authority,
            audition_continuity_outcome,
            transform_scheduling_posture,
            transform_scheduling_authority,
            transform_scheduling_outcome,
            queued_preview_request_count,
            previewable_asset_count,
            active_audition_clip_count,
            pending_transform_clip_count: pending_clip_count,
            ready_transform_clip_count: ready_clip_count,
            fallback_transform_clip_count: fallback_clip_count,
        }
    }
}
