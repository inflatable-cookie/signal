use super::super::*;

pub(super) fn validate_offline_preview_request(
    request: &RuntimeOfflineRenderRequest,
) -> Result<(), RuntimeError> {
    if request.request_id.trim().is_empty() {
        return Err(RuntimeError::new(
            RuntimeErrorKind::InvalidRequest,
            "offline render requests require a non-empty request id",
        ));
    }
    if request.duration_samples == 0 {
        return Err(RuntimeError::new(
            RuntimeErrorKind::InvalidRequest,
            "offline render requests require a non-zero duration",
        ));
    }
    if request.export_sample_rate_hz == 0 {
        return Err(RuntimeError::new(
            RuntimeErrorKind::InvalidRequest,
            "offline render requests require a positive export sample rate",
        ));
    }

    Ok(())
}
