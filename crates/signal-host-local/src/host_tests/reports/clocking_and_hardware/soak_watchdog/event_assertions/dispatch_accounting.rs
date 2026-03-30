use super::super::super::super::super::*;

pub(super) fn assert_mixed_watchdog_dispatch_accounting(
    _summary: &LocalRuntimeHostSummary,
    supervisor: &RuntimeHostSupervisorReport,
) {
    assert_eq!(supervisor.block_dispatch_event_count(), 28);
    assert_eq!(supervisor.lease_rollover_event_count(), 2);
    assert_eq!(supervisor.invalidation_event_count(), 6);
    assert_eq!(supervisor.completion_slot_event_count(), 45);
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::BlockDispatch {
                    stage: BlockDispatchStage::Requested,
                    ..
                }
            ))
            .count(),
        14
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::BlockDispatch {
                    stage: BlockDispatchStage::TimedOut,
                    ..
                }
            ))
            .count(),
        2
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::BrokerInvalidation {
                    stage: BrokerInvalidationStage::CompletionRegionInvalidated,
                    ..
                }
            ))
            .count(),
        3
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::BrokerInvalidation {
                    stage: BrokerInvalidationStage::LeaseEpochInvalidated,
                    ..
                }
            ))
            .count(),
        3
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::CompletionSlotTransition {
                    stage: CompletionSlotStage::ReadyForProcessing,
                    ..
                }
            ))
            .count(),
        14
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::CompletionSlotTransition {
                    stage: CompletionSlotStage::Processing,
                    ..
                }
            ))
            .count(),
        12
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::CompletionSlotTransition {
                    stage: CompletionSlotStage::Completed,
                    ..
                }
            ))
            .count(),
        12
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::CompletionSlotTransition {
                    stage: CompletionSlotStage::TimedOut,
                    ..
                }
            ))
            .count(),
        2
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::CompletionSlotTransition {
                    stage: CompletionSlotStage::FallbackApplied,
                    ..
                }
            ))
            .count(),
        2
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::CompletionSlotTransition {
                    stage: CompletionSlotStage::Invalidated,
                    ..
                }
            ))
            .count(),
        3
    );
}
