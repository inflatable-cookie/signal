use signal_runtime::{BrokerFailureStage, RuntimeError, SignalRuntime};

pub(crate) fn runtime_error_from_io(error: std::io::Error) -> RuntimeError {
    RuntimeError {
        kind: signal_runtime::RuntimeErrorKind::ResourceUnavailable,
        message: error.to_string(),
    }
}

pub(crate) fn record_broker_failure_and_convert(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: Option<String>,
    processing_epoch: Option<u64>,
    block_sequence: Option<u64>,
    stage: BrokerFailureStage,
    error: std::io::Error,
) -> RuntimeError {
    let detail = error.to_string();
    runtime.record_broker_failure(
        sandbox_id,
        lease_id,
        processing_epoch,
        block_sequence,
        stage,
        detail.clone(),
    );
    RuntimeError {
        kind: signal_runtime::RuntimeErrorKind::ResourceUnavailable,
        message: detail,
    }
}
