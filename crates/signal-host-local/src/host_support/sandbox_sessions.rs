//! Sandbox session plumbing for discovered plugin types.
//!
//! When the out-of-process broker is enabled
//! (`SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND`), ensuring a sandbox spawns the
//! broker child process and drives it over stdio: attach (allocating a real
//! file-backed shared-memory lease), a verified shared-memory block exercise,
//! and a bounded timeout probe. Without the broker, only the discovered
//! metadata is recorded — no instantiation or execution is claimed.

use signal_plugin::PluginIoLayout;
use signal_runtime::{
    ensure_prepared_sandbox_session, PluginSandboxSpec, PreparedBrokerSandboxSpec,
    PreparedSandboxSessionRecord, RuntimeError, SandboxBrokerSpawnConfig, SignalRuntime,
};

pub(crate) use signal_runtime::{teardown_broker_sandbox_session, SandboxBrokerSession};

/// Ensures a sandbox session for a plugin type found in the last explicit
/// scan. `format_tag` is a short token (`clap`/`au`/`vst3`) used in instance
/// IDs and summaries; `extra_env` carries the discovered bundle/module/library
/// location into the broker child's environment.
pub(crate) fn ensure_discovered_sandbox_session(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    format_tag: &str,
    plugin_type_id: &str,
    default_io_layout: PluginIoLayout,
    extra_env: Vec<(String, String)>,
) -> Result<Option<SandboxBrokerSession>, RuntimeError> {
    let instance_id = format!("instance:local:{format_tag}:{}", request.sandbox_id);
    let plugin_type_id = plugin_type_id.to_string();
    let direct_instance_id = instance_id.clone();
    ensure_prepared_sandbox_session(
        runtime,
        request,
        PreparedBrokerSandboxSpec {
            plugin_type_id: plugin_type_id.clone(),
            default_io_layout,
            fallback_instance_id: instance_id,
            spawn_config: SandboxBrokerSpawnConfig {
                env: extra_env,
                read_timeout_ms: None,
            },
        },
        |_runtime| Ok((None, None)),
        |runtime| {
            Ok(PreparedSandboxSessionRecord {
                plugin_type_id: plugin_type_id.clone(),
                instance_id: direct_instance_id.clone(),
                sample_rate_hz: runtime.config().sample_rate.0,
                max_block_frames: runtime.config().graph.block_size as u32,
                audio_inputs: default_io_layout.audio_inputs,
                audio_outputs: default_io_layout.audio_outputs,
                midi_inputs: default_io_layout.midi_inputs,
                midi_outputs: default_io_layout.midi_outputs,
                processing_epoch: None,
                lease_id: format!("lease:{}", request.sandbox_id),
                region_id: format!("region:{}", request.sandbox_id),
                summary: Some(format!(
                    "{format_tag}:discovered plugin_type={plugin_type_id} io={default_io_layout:?} (metadata only; no instance)"
                )),
            })
        },
        |runtime, request, session| {
            let execution = session
                .client
                .request_execution_stream()
                .map_err(|error| broker_exercise_error(runtime, request, session, error))?;
            let timeout_probe = session
                .client
                .request_timeout_probe()
                .map_err(|error| broker_exercise_error(runtime, request, session, error))?;
            signal_runtime::record_broker_attached_execution_summary(
                runtime,
                request,
                session,
                format!(
                    "broker:{} | broker:{}",
                    execution.detail, timeout_probe.detail
                ),
            );
            Ok(())
        },
    )
}

fn broker_exercise_error(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    session: &SandboxBrokerSession,
    error: std::io::Error,
) -> RuntimeError {
    runtime.record_broker_failure(
        request.sandbox_id.as_str(),
        Some(session.attached.lease_id.clone()),
        Some(session.attached.processing_epoch),
        None,
        signal_runtime::BrokerFailureStage::PayloadWrite,
        error.to_string(),
    );
    RuntimeError::new(
        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
        format!("sandbox broker exercise failed: {error}"),
    )
}
