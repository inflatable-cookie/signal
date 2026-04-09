use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use signal_plugin::PluginIoLayout;

use crate::{
    BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, RecoveryRestartIntent, RuntimeError, RuntimeErrorKind,
    RuntimeLv2PreparedNegotiationRecord, SignalRuntime, StopReason,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxBrokerAttachedSession {
    pub sandbox_id: String,
    pub instance_id: String,
    pub processing_epoch: u64,
    pub lease_id: String,
    pub region_id: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SandboxBrokerReceiptLine {
    state: String,
    sandbox_id: String,
    instance_id: Option<String>,
    processing_epoch: Option<u64>,
    lease_id: Option<String>,
    region_id: Option<String>,
    detail: String,
}

pub struct SandboxBrokerClientSession {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxBrokerFlavor {
    Demo,
    Au,
    Lv2,
    Vst3,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SandboxBrokerSpawnConfig {
    pub env: Vec<(String, String)>,
}

pub struct SandboxBrokerSession {
    pub client: SandboxBrokerClientSession,
    pub attached: SandboxBrokerAttachedSession,
    pub flavor: SandboxBrokerFlavor,
    pub prepared_summary: Option<String>,
    pub teardown_summary: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxBrokerExecutionSummary {
    pub processed_blocks: usize,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedSandboxSessionRecord {
    pub plugin_type_id: String,
    pub instance_id: String,
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub audio_inputs: u16,
    pub audio_outputs: u16,
    pub midi_inputs: u16,
    pub midi_outputs: u16,
    pub processing_epoch: Option<u64>,
    pub lease_id: String,
    pub region_id: String,
    pub lv2_prepared_negotiation: Option<RuntimeLv2PreparedNegotiationRecord>,
    pub summary: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedBrokerSandboxSpec {
    pub plugin_type_id: String,
    pub default_io_layout: PluginIoLayout,
    pub fallback_instance_id: String,
    pub flavor: SandboxBrokerFlavor,
    pub spawn_config: SandboxBrokerSpawnConfig,
    pub lv2_prepared_negotiation: Option<RuntimeLv2PreparedNegotiationRecord>,
}

impl SandboxBrokerClientSession {
    pub fn broker_enabled() -> bool {
        std::env::var_os("SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND").is_some()
    }

    pub fn spawn_from_env(config: &SandboxBrokerSpawnConfig) -> Result<Self, RuntimeError> {
        let command = std::env::var("SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND").map_err(|_| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                "missing SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND",
            )
        })?;
        let args = std::env::var("SIGNAL_PLUGIN_SANDBOX_BROKER_ARGS")
            .ok()
            .map(|value| {
                value
                    .split_whitespace()
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let mut process = Command::new(&command);
        process
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(workdir) = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_BROKER_WORKDIR") {
            process.current_dir(PathBuf::from(workdir));
        }
        for (key, value) in &config.env {
            process.env(key, value);
        }
        let mut child = process.spawn().map_err(io_runtime_error)?;

        let stdin = child.stdin.take().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                "sandbox broker missing stdin pipe",
            )
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                "sandbox broker missing stdout pipe",
            )
        })?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    pub fn read_startup_receipts(&mut self) -> Result<(), RuntimeError> {
        let starting = self.read_receipt().map_err(io_runtime_error)?;
        let ready = self.read_receipt().map_err(io_runtime_error)?;
        if starting.state != "starting" || ready.state != "ready" {
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!(
                    "unexpected broker startup sequence: {} then {}",
                    starting.state, ready.state
                ),
            ));
        }
        Ok(())
    }

    pub fn attach(
        &mut self,
        flavor: SandboxBrokerFlavor,
        fallback_sandbox_id: &str,
        fallback_instance_id: &str,
    ) -> std::io::Result<SandboxBrokerAttachedSession> {
        self.write_command(attach_command_for(flavor))?;
        let attached = self.read_receipt()?;
        if attached.state != "attached" {
            return Err(std::io::Error::other(format!(
                "unexpected broker attach state: {} ({})",
                attached.state, attached.detail
            )));
        }

        Ok(SandboxBrokerAttachedSession {
            sandbox_id: attached.sandbox_id,
            instance_id: attached
                .instance_id
                .unwrap_or_else(|| fallback_instance_id.to_string()),
            processing_epoch: attached.processing_epoch.unwrap_or(1),
            lease_id: attached
                .lease_id
                .unwrap_or_else(|| format!("lease:{fallback_sandbox_id}")),
            region_id: attached
                .region_id
                .unwrap_or_else(|| format!("region:{fallback_sandbox_id}")),
            detail: attached.detail,
        })
    }

    pub fn request_teardown(
        &mut self,
        flavor: SandboxBrokerFlavor,
    ) -> std::io::Result<(
        String,
        Option<String>,
        Option<u64>,
        Option<String>,
        Option<String>,
        String,
    )> {
        self.write_command(teardown_command_for(flavor))?;
        let teardown = self.read_receipt()?;
        Ok((
            teardown.state,
            teardown.instance_id,
            teardown.processing_epoch,
            teardown.lease_id,
            teardown.region_id,
            teardown.detail,
        ))
    }

    pub fn request_vst3_execution_stream(
        &mut self,
    ) -> std::io::Result<SandboxBrokerExecutionSummary> {
        self.write_command("stream-vst3")?;
        let mut processed_blocks = 0usize;

        loop {
            let receipt = self.read_receipt()?;
            match receipt.state.as_str() {
                "running" => {
                    processed_blocks += 1;
                }
                "attached" => {
                    return Ok(SandboxBrokerExecutionSummary {
                        processed_blocks,
                        detail: receipt.detail,
                    });
                }
                "crashed" => {
                    return Err(std::io::Error::other(format!(
                        "sandbox broker execution stream crashed: {}",
                        receipt.detail
                    )));
                }
                other => {
                    return Err(std::io::Error::other(format!(
                        "unexpected broker execution stream state: {} ({})",
                        other, receipt.detail
                    )));
                }
            }
        }
    }

    pub fn request_vst3_refresh(&mut self) -> std::io::Result<SandboxBrokerExecutionSummary> {
        self.write_command("refresh-vst3")?;
        let receipt = self.read_receipt()?;
        match receipt.state.as_str() {
            "attached" => Ok(SandboxBrokerExecutionSummary {
                processed_blocks: 0,
                detail: receipt.detail,
            }),
            "crashed" => Err(std::io::Error::other(format!(
                "sandbox broker refresh crashed: {}",
                receipt.detail
            ))),
            other => Err(std::io::Error::other(format!(
                "unexpected broker refresh state: {} ({})",
                other, receipt.detail
            ))),
        }
    }

    pub fn request_vst3_timeout(&mut self) -> std::io::Result<SandboxBrokerExecutionSummary> {
        self.write_command("timeout-vst3")?;
        let receipt = self.read_receipt()?;
        match receipt.state.as_str() {
            "attached" => Ok(SandboxBrokerExecutionSummary {
                processed_blocks: 0,
                detail: receipt.detail,
            }),
            "crashed" => Err(std::io::Error::other(format!(
                "sandbox broker timeout path crashed: {}",
                receipt.detail
            ))),
            other => Err(std::io::Error::other(format!(
                "unexpected broker timeout state: {} ({})",
                other, receipt.detail
            ))),
        }
    }

    pub fn request_lv2_execution_stream(
        &mut self,
    ) -> std::io::Result<SandboxBrokerExecutionSummary> {
        self.write_command("stream-lv2")?;
        let mut processed_blocks = 0usize;

        loop {
            let receipt = self.read_receipt()?;
            match receipt.state.as_str() {
                "running" => {
                    processed_blocks += 1;
                }
                "attached" => {
                    return Ok(SandboxBrokerExecutionSummary {
                        processed_blocks,
                        detail: receipt.detail,
                    });
                }
                "crashed" => {
                    return Err(std::io::Error::other(format!(
                        "sandbox broker lv2 execution stream crashed: {}",
                        receipt.detail
                    )));
                }
                other => {
                    return Err(std::io::Error::other(format!(
                        "unexpected broker lv2 execution stream state: {} ({})",
                        other, receipt.detail
                    )));
                }
            }
        }
    }

    pub fn shutdown(mut self) -> std::io::Result<()> {
        self.write_command("shutdown")?;
        match self.read_receipt() {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {}
            Err(error) => return Err(error),
        }
        let status = self.child.wait()?;
        if !status.success() {
            return Err(std::io::Error::other(
                "sandbox broker exited unsuccessfully",
            ));
        }
        Ok(())
    }

    fn read_receipt(&mut self) -> std::io::Result<SandboxBrokerReceiptLine> {
        let mut line = String::new();
        let bytes = self.stdout.read_line(&mut line)?;
        if bytes == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "sandbox broker closed stdout",
            ));
        }
        parse_broker_receipt_line(&line)
    }

    fn write_command(&mut self, command: &str) -> std::io::Result<()> {
        writeln!(self.stdin, "{command}")?;
        self.stdin.flush()
    }
}

fn attach_command_for(flavor: SandboxBrokerFlavor) -> &'static str {
    match flavor {
        SandboxBrokerFlavor::Demo => "attach-demo",
        SandboxBrokerFlavor::Au => "attach-au",
        SandboxBrokerFlavor::Lv2 => "attach-lv2",
        SandboxBrokerFlavor::Vst3 => "attach-vst3",
    }
}

fn teardown_command_for(flavor: SandboxBrokerFlavor) -> &'static str {
    match flavor {
        SandboxBrokerFlavor::Demo => "teardown-demo",
        SandboxBrokerFlavor::Au => "teardown-au",
        SandboxBrokerFlavor::Lv2 => "teardown-lv2",
        SandboxBrokerFlavor::Vst3 => "teardown-vst3",
    }
}

pub fn record_broker_sandbox_prepared(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    record: PreparedSandboxSessionRecord,
) {
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::SandboxHandshaken,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::PluginTypeLoaded,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::InstanceCreated,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::InstancePrepared,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: request.sandbox_id.clone(),
        plugin_type_id: record.plugin_type_id,
        instance_id: record.instance_id,
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: record.processing_epoch,
        processing_sample_rate_hz: Some(record.sample_rate_hz),
        processing_max_block_frames: Some(record.max_block_frames),
        audio_inputs: Some(record.audio_inputs),
        audio_outputs: Some(record.audio_outputs),
        midi_inputs: Some(record.midi_inputs),
        midi_outputs: Some(record.midi_outputs),
        last_fault: None,
    });
    if let Some(negotiation) = record.lv2_prepared_negotiation {
        runtime.record_plugin_sandbox_lv2_prepared_negotiation(
            request.sandbox_id.as_str(),
            negotiation,
        );
    }
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::TransportAttached,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_transport(
        request.sandbox_id.as_str(),
        record.lease_id,
        record.region_id,
        PluginSandboxTransportStage::Attached,
        record.processing_epoch,
        record.summary,
    );
}

pub fn ensure_broker_sandbox_session(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    plugin_type_id: &str,
    default_io_layout: PluginIoLayout,
    fallback_instance_id: &str,
    flavor: SandboxBrokerFlavor,
    spawn_config: SandboxBrokerSpawnConfig,
    prepared_summary: Option<String>,
    teardown_summary: Option<String>,
    lv2_prepared_negotiation: Option<RuntimeLv2PreparedNegotiationRecord>,
) -> Result<SandboxBrokerSession, RuntimeError> {
    let mut client = SandboxBrokerClientSession::spawn_from_env(&spawn_config)?;
    client.read_startup_receipts()?;
    let attached = client
        .attach(flavor, request.sandbox_id.as_str(), fallback_instance_id)
        .map_err(|error| {
            record_broker_failure_and_convert(
                runtime,
                request.sandbox_id.as_str(),
                None,
                None,
                None,
                BrokerFailureStage::PreparePlanCreate,
                error,
            )
        })?;

    record_broker_sandbox_prepared(
        runtime,
        request,
        PreparedSandboxSessionRecord {
            plugin_type_id: plugin_type_id.to_string(),
            instance_id: attached.instance_id.clone(),
            sample_rate_hz: runtime.config().sample_rate.0,
            max_block_frames: runtime.config().graph.block_size as u32,
            audio_inputs: default_io_layout.audio_inputs,
            audio_outputs: default_io_layout.audio_outputs,
            midi_inputs: default_io_layout.midi_inputs,
            midi_outputs: default_io_layout.midi_outputs,
            processing_epoch: Some(attached.processing_epoch),
            lease_id: attached.lease_id.clone(),
            region_id: attached.region_id.clone(),
            lv2_prepared_negotiation,
            summary: Some(match &prepared_summary {
                Some(summary) => format!("broker:{} | {}", attached.detail, summary),
                None => format!("broker:{}", attached.detail),
            }),
        },
    );

    Ok(SandboxBrokerSession {
        client,
        attached,
        flavor,
        prepared_summary,
        teardown_summary,
    })
}

pub fn ensure_prepared_sandbox_session<BrokerPrepareFn, DirectPrepareFn, AfterBrokerAttachFn>(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    broker_spec: PreparedBrokerSandboxSpec,
    broker_prepare: BrokerPrepareFn,
    direct_prepare: DirectPrepareFn,
    after_broker_attach: AfterBrokerAttachFn,
) -> Result<Option<SandboxBrokerSession>, RuntimeError>
where
    BrokerPrepareFn:
        FnOnce(&mut SignalRuntime) -> Result<(Option<String>, Option<String>), RuntimeError>,
    DirectPrepareFn:
        FnOnce(&mut SignalRuntime) -> Result<PreparedSandboxSessionRecord, RuntimeError>,
    AfterBrokerAttachFn: FnOnce(
        &mut SignalRuntime,
        &PluginSandboxSpec,
        &mut SandboxBrokerSession,
    ) -> Result<(), RuntimeError>,
{
    if SandboxBrokerClientSession::broker_enabled() {
        let (prepared_summary, teardown_summary) = broker_prepare(runtime)?;
        let mut broker_session = ensure_broker_sandbox_session(
            runtime,
            request,
            broker_spec.plugin_type_id.as_str(),
            broker_spec.default_io_layout,
            broker_spec.fallback_instance_id.as_str(),
            broker_spec.flavor,
            broker_spec.spawn_config,
            prepared_summary,
            teardown_summary,
            broker_spec.lv2_prepared_negotiation,
        )?;
        after_broker_attach(runtime, request, &mut broker_session)?;
        Ok(Some(broker_session))
    } else {
        let record = direct_prepare(runtime)?;
        record_broker_sandbox_prepared(runtime, request, record);
        Ok(None)
    }
}

pub fn record_protocol_violation_prepare_failure(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    plugin_type_id: String,
    instance_id: String,
    default_io_layout: PluginIoLayout,
    lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    detail: String,
) -> RuntimeError {
    if let Some(stage) = lifecycle_stage {
        runtime.record_plugin_sandbox_lifecycle(request.sandbox_id.as_str(), stage, None);
    } else {
        runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::PluginTypeLoaded,
            None,
        );
    }
    runtime.record_plugin_sandbox_fault(
        request.sandbox_id.as_str(),
        crate::PluginFaultKind::ProtocolViolation,
        detail.clone(),
        None,
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: request.sandbox_id.clone(),
        plugin_type_id,
        instance_id,
        lifecycle_state: "Faulted".into(),
        readiness_state: "Faulted".into(),
        degraded_reasons: vec![detail.clone()],
        active: false,
        processing_epoch: None,
        processing_sample_rate_hz: Some(runtime.config().sample_rate.0),
        processing_max_block_frames: Some(runtime.config().graph.block_size as u32),
        audio_inputs: Some(default_io_layout.audio_inputs),
        audio_outputs: Some(default_io_layout.audio_outputs),
        midi_inputs: Some(default_io_layout.midi_inputs),
        midi_outputs: Some(default_io_layout.midi_outputs),
        last_fault: Some(crate::PluginSandboxInstanceFaultRecord {
            kind: "ProtocolViolation".into(),
            severity: "Error".into(),
            message: detail.clone(),
        }),
    });
    RuntimeError::new(RuntimeErrorKind::InvalidRequest, detail)
}

pub fn record_broker_attached_execution_summary(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    session: &mut SandboxBrokerSession,
    execution_summary: String,
) {
    runtime.record_plugin_sandbox_transport(
        request.sandbox_id.as_str(),
        session.attached.lease_id.as_str(),
        session.attached.region_id.as_str(),
        PluginSandboxTransportStage::Attached,
        Some(session.attached.processing_epoch),
        Some(execution_summary.clone()),
    );
    session.prepared_summary = Some(match session.prepared_summary.take() {
        Some(summary) => format!("{summary} | {execution_summary}"),
        None => execution_summary,
    });
}

pub fn run_vst3_broker_execution_sequence(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    session: &mut SandboxBrokerSession,
) -> Result<(), RuntimeError> {
    let first_execution = session
        .client
        .request_vst3_execution_stream()
        .map_err(|error| {
            record_broker_failure_and_convert(
                runtime,
                request.sandbox_id.as_str(),
                Some(session.attached.lease_id.clone()),
                Some(session.attached.processing_epoch),
                None,
                BrokerFailureStage::PreparePlanCreate,
                error,
            )
        })?;
    let second_execution = session
        .client
        .request_vst3_execution_stream()
        .map_err(|error| {
            record_broker_failure_and_convert(
                runtime,
                request.sandbox_id.as_str(),
                Some(session.attached.lease_id.clone()),
                Some(session.attached.processing_epoch),
                None,
                BrokerFailureStage::PreparePlanCreate,
                error,
            )
        })?;
    let refresh = session.client.request_vst3_refresh().map_err(|error| {
        record_broker_failure_and_convert(
            runtime,
            request.sandbox_id.as_str(),
            Some(session.attached.lease_id.clone()),
            Some(session.attached.processing_epoch),
            None,
            BrokerFailureStage::PreparePlanCreate,
            error,
        )
    })?;
    let refreshed_execution = session
        .client
        .request_vst3_execution_stream()
        .map_err(|error| {
            record_broker_failure_and_convert(
                runtime,
                request.sandbox_id.as_str(),
                Some(session.attached.lease_id.clone()),
                Some(session.attached.processing_epoch),
                None,
                BrokerFailureStage::PreparePlanCreate,
                error,
            )
        })?;
    let timeout = session.client.request_vst3_timeout().map_err(|error| {
        record_broker_failure_and_convert(
            runtime,
            request.sandbox_id.as_str(),
            Some(session.attached.lease_id.clone()),
            Some(session.attached.processing_epoch),
            None,
            BrokerFailureStage::PreparePlanCreate,
            error,
        )
    })?;
    record_broker_attached_execution_summary(
        runtime,
        request,
        session,
        format!(
            "broker:{} | broker:{} | broker:{} | broker:{} | broker:{}",
            first_execution.detail,
            second_execution.detail,
            refresh.detail,
            refreshed_execution.detail,
            timeout.detail
        ),
    );
    Ok(())
}

pub fn begin_recovery_overlap(runtime: &mut SignalRuntime) {
    runtime.set_active_plugin_sandboxes(2);
}

pub fn complete_recovery_overlap_restart(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: Option<&str>,
    region_id: Option<&str>,
) {
    runtime.set_active_plugin_sandboxes(1);
    if let (Some(lease_id), Some(region_id)) = (lease_id, region_id) {
        runtime.promote_transport_session_to_steady_state(sandbox_id, lease_id, region_id);
    }
}

pub fn rollback_recovery_overlap(runtime: &mut SignalRuntime) {
    runtime.set_active_plugin_sandboxes(0);
}

pub fn begin_brokered_recovery_cycle<InvalidateFn>(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    processing_epoch: u64,
    last_block_sequence: u64,
    intent: RecoveryRestartIntent,
    mut invalidate_active_epoch: InvalidateFn,
) where
    InvalidateFn: FnMut(u64) -> (bool, bool),
{
    runtime.record_recovery_cycle(
        sandbox_id,
        intent,
        StopReason::DegradedModeRecovery,
        Some(processing_epoch),
    );
    let (completion_invalidated, lease_invalidated) = invalidate_active_epoch(processing_epoch);
    let recovery_reason = match intent {
        RecoveryRestartIntent::CrashRecovery => "crash recovery teardown",
        RecoveryRestartIntent::WatchdogRecovery => "watchdog recovery teardown",
    };
    if completion_invalidated {
        runtime.record_completion_slot_transition(
            sandbox_id,
            lease_id,
            processing_epoch,
            last_block_sequence,
            CompletionSlotStage::Invalidated,
        );
        runtime.record_broker_invalidation(
            sandbox_id,
            lease_id,
            processing_epoch,
            Some(last_block_sequence),
            BrokerInvalidationStage::CompletionRegionInvalidated,
            recovery_reason,
        );
    }
    if lease_invalidated {
        runtime.record_broker_invalidation(
            sandbox_id,
            lease_id,
            processing_epoch,
            Some(last_block_sequence),
            BrokerInvalidationStage::LeaseEpochInvalidated,
            recovery_reason,
        );
    }
}

pub fn handle_overlap_prepare_contention(
    requested: bool,
    competing_attach_result: Result<(), RuntimeError>,
) -> Result<(), RuntimeError> {
    if !requested {
        return Ok(());
    }

    Err(match competing_attach_result {
        Ok(()) => RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            "expected overlapping replacement attach contention",
        ),
        Err(error) => error,
    })
}

pub fn complete_recovery_overlap_restart_or_rollback(
    restart_result: Result<(), RuntimeError>,
    inject_replacement_start_failure: bool,
    start_result: Option<Result<(), RuntimeError>>,
) -> Result<(), RuntimeError> {
    if let Err(error) = restart_result {
        return Err(error);
    }

    if inject_replacement_start_failure {
        return Err(RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            "injected replacement start failure during overlap recovery",
        ));
    }

    if let Some(Err(error)) = start_result {
        return Err(error);
    }
    Ok(())
}

pub fn complete_lingering_recovery_restart_or_rollback(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    restart_result: Result<(), RuntimeError>,
    replacement_transport: Option<(&str, &str)>,
    start_result: Result<(), RuntimeError>,
) -> Result<(), RuntimeError> {
    restart_result?;
    complete_recovery_overlap_restart(runtime, sandbox_id, None, None);

    let (lease_id, region_id) = replacement_transport.unwrap_or_default();
    complete_recovery_overlap_restart(
        runtime,
        sandbox_id,
        (!lease_id.is_empty()).then_some(lease_id),
        (!region_id.is_empty()).then_some(region_id),
    );

    if let Err(error) = start_result {
        rollback_recovery_overlap(runtime);
        return Err(error);
    }

    Ok(())
}

pub enum RecoveryOverlapOldTransportTeardownOutcome {
    Continue,
    RollbackKeepReplacement(RuntimeError),
    RollbackClearOverlap(RuntimeError),
}

pub fn handle_recovery_overlap_old_transport_teardown(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    last_block_sequence: u64,
    deferred_teardown_failure: bool,
    destroy_result: Result<(), String>,
    injected_old_transport_teardown_failure: bool,
    transport_teardown_result: Result<(), String>,
) -> RecoveryOverlapOldTransportTeardownOutcome {
    let detail = "recovery overlap old transport teardown";
    record_broker_transport_detach_requested(
        runtime,
        sandbox_id,
        lease_id,
        region_id,
        processing_epoch,
        detail,
    );

    if deferred_teardown_failure {
        let error = std::io::Error::other("deferred old transport teardown during recovery retry");
        record_broker_transport_detach_failure(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            Some(last_block_sequence),
            crate::BrokerFailureStage::TransportTeardown,
            error.to_string(),
        );
        return RecoveryOverlapOldTransportTeardownOutcome::RollbackKeepReplacement(
            io_runtime_error(error),
        );
    }

    if let Err(error) = destroy_result {
        record_broker_transport_detach_failure(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            Some(last_block_sequence),
            crate::BrokerFailureStage::TransportDestroy,
            error,
        );
        runtime.end_transport_session(sandbox_id, lease_id, region_id);
        return RecoveryOverlapOldTransportTeardownOutcome::RollbackClearOverlap(
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                "failed to destroy recovery overlap old transport region",
            ),
        );
    }

    if injected_old_transport_teardown_failure {
        let error = std::io::Error::other(
            "injected old transport teardown failure during overlap recovery",
        );
        record_broker_transport_detach_failure(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            Some(last_block_sequence),
            crate::BrokerFailureStage::TransportTeardown,
            error.to_string(),
        );
        runtime.end_transport_session(sandbox_id, lease_id, region_id);
        return RecoveryOverlapOldTransportTeardownOutcome::RollbackClearOverlap(io_runtime_error(
            error,
        ));
    }

    if let Err(error) = transport_teardown_result {
        record_broker_transport_detach_failure(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            Some(last_block_sequence),
            crate::BrokerFailureStage::TransportTeardown,
            error,
        );
        runtime.end_transport_session(sandbox_id, lease_id, region_id);
        return RecoveryOverlapOldTransportTeardownOutcome::RollbackClearOverlap(
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                "failed to tear down recovery overlap old transport",
            ),
        );
    }

    record_broker_sandbox_detached(
        runtime,
        sandbox_id,
        lease_id,
        region_id,
        processing_epoch,
        detail,
        false,
    );
    runtime.end_transport_session(sandbox_id, lease_id, region_id);
    RecoveryOverlapOldTransportTeardownOutcome::Continue
}

pub fn record_broker_transport_detach_requested(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    detail: impl Into<String>,
) {
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        lease_id,
        region_id,
        PluginSandboxTransportStage::DetachRequested,
        Some(processing_epoch),
        Some(detail.into()),
    );
}

pub fn record_broker_transport_detach_fault(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    detail: impl Into<String>,
) {
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        lease_id,
        region_id,
        PluginSandboxTransportStage::DetachFault,
        Some(processing_epoch),
        Some(detail.into()),
    );
}

pub fn record_broker_transport_detach_failure(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    last_block_sequence: Option<u64>,
    stage: crate::BrokerFailureStage,
    detail: impl Into<String>,
) {
    let detail = detail.into();
    runtime.record_broker_failure(
        sandbox_id,
        Some(lease_id.to_string()),
        Some(processing_epoch),
        last_block_sequence,
        stage,
        detail.clone(),
    );
    record_broker_transport_detach_fault(
        runtime,
        sandbox_id,
        lease_id,
        region_id,
        processing_epoch,
        detail,
    );
}

pub fn record_broker_sandbox_detached(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    detail: impl Into<String>,
    record_instance_destroyed: bool,
) {
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        lease_id,
        region_id,
        PluginSandboxTransportStage::Detached,
        Some(processing_epoch),
        Some(detail.into()),
    );
    runtime.record_plugin_sandbox_lifecycle(
        sandbox_id,
        PluginSandboxLifecycleStage::TransportTornDown,
        Some(processing_epoch),
    );
    if record_instance_destroyed {
        runtime.record_plugin_sandbox_lifecycle(
            sandbox_id,
            PluginSandboxLifecycleStage::InstanceDestroyed,
            Some(processing_epoch),
        );
    }
}

pub fn complete_broker_transport_detach(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    detail: impl Into<String>,
    record_instance_destroyed: bool,
) {
    record_broker_sandbox_detached(
        runtime,
        sandbox_id,
        lease_id,
        region_id,
        processing_epoch,
        detail,
        record_instance_destroyed,
    );
    runtime.end_transport_session(sandbox_id, lease_id, region_id);
}

pub fn teardown_broker_sandbox_session(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    mut session: SandboxBrokerSession,
) -> Result<(), RuntimeError> {
    record_broker_transport_detach_requested(
        runtime,
        sandbox_id,
        session.attached.lease_id.as_str(),
        session.attached.region_id.as_str(),
        session.attached.processing_epoch,
        "broker_teardown_requested",
    );

    let (state, _instance_id, _epoch, _lease_id, _region_id, detail) = session
        .client
        .request_teardown(session.flavor)
        .map_err(|error| {
            record_broker_failure_and_convert(
                runtime,
                sandbox_id,
                Some(session.attached.lease_id.clone()),
                Some(session.attached.processing_epoch),
                None,
                BrokerFailureStage::TransportTeardown,
                error,
            )
        })?;
    if state != "teardown_complete" {
        return Err(record_broker_failure_and_convert(
            runtime,
            sandbox_id,
            Some(session.attached.lease_id.clone()),
            Some(session.attached.processing_epoch),
            None,
            BrokerFailureStage::TransportTeardown,
            std::io::Error::other(format!(
                "unexpected broker teardown state: {state} ({detail})"
            )),
        ));
    }

    let detail = match &session.teardown_summary {
        Some(teardown_summary) => format!("{detail} | {teardown_summary}"),
        None => detail,
    };
    record_broker_sandbox_detached(
        runtime,
        sandbox_id,
        session.attached.lease_id.as_str(),
        session.attached.region_id.as_str(),
        session.attached.processing_epoch,
        detail,
        true,
    );

    session.client.shutdown().map_err(|error| {
        record_broker_failure_and_convert(
            runtime,
            sandbox_id,
            Some(session.attached.lease_id.clone()),
            Some(session.attached.processing_epoch),
            None,
            BrokerFailureStage::TransportTeardown,
            error,
        )
    })?;
    Ok(())
}

pub fn finalize_brokered_recovery_transport_detach(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    last_block_sequence: u64,
    detail: &str,
    record_instance_destroyed: bool,
    destroy_error: Option<String>,
    teardown_error: Option<String>,
) {
    record_broker_transport_detach_requested(
        runtime,
        sandbox_id,
        lease_id,
        region_id,
        processing_epoch,
        detail,
    );

    let destroy_failed = destroy_error.is_some();
    let teardown_failed = teardown_error.is_some();

    if let Some(error) = destroy_error {
        record_broker_transport_detach_failure(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            Some(last_block_sequence),
            crate::BrokerFailureStage::TransportDestroy,
            error,
        );
    }

    if let Some(error) = teardown_error {
        record_broker_transport_detach_failure(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            Some(last_block_sequence),
            crate::BrokerFailureStage::TransportTeardown,
            error,
        );
    }

    if !destroy_failed && !teardown_failed {
        complete_broker_transport_detach(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            detail,
            record_instance_destroyed,
        );
    }
}

fn parse_broker_receipt_line(line: &str) -> std::io::Result<SandboxBrokerReceiptLine> {
    let mut state = None;
    let mut sandbox_id = None;
    let mut instance_id = None;
    let mut processing_epoch = None;
    let mut lease_id = None;
    let mut region_id = None;
    let mut detail = None;

    for token in line.split_whitespace().skip(1) {
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };
        match key {
            "state" => state = Some(value.to_string()),
            "sandbox_id" => sandbox_id = Some(value.to_string()),
            "instance_id" if value != "-" => instance_id = Some(value.to_string()),
            "epoch" if value != "-" => processing_epoch = value.parse::<u64>().ok(),
            "lease_id" if value != "-" => lease_id = Some(value.to_string()),
            "region_id" if value != "-" => region_id = Some(value.to_string()),
            "detail" => detail = Some(value.to_string()),
            _ => {}
        }
    }

    Ok(SandboxBrokerReceiptLine {
        state: state.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "broker receipt missing state",
            )
        })?,
        sandbox_id: sandbox_id.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "broker receipt missing sandbox_id",
            )
        })?,
        instance_id,
        processing_epoch,
        lease_id,
        region_id,
        detail: detail.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "broker receipt missing detail",
            )
        })?,
    })
}

fn io_runtime_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(RuntimeErrorKind::ResourceUnavailable, error.to_string())
}

fn record_broker_failure_and_convert(
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
    RuntimeError::new(RuntimeErrorKind::ResourceUnavailable, detail)
}

#[cfg(test)]
mod tests {
    use super::parse_broker_receipt_line;

    #[test]
    fn parses_broker_receipt_lines() {
        let receipt = parse_broker_receipt_line(
            "signal-plugin-sandbox state=attached sandbox_id=plugin-sandbox-broker instance_id=instance:sandbox:default epoch=1 lease_id=lease:plugin-sandbox-broker region_id=region:plugin-sandbox-broker detail=lease_attached\n",
        )
        .expect("receipt should parse");

        assert_eq!(receipt.state, "attached");
        assert_eq!(receipt.sandbox_id, "plugin-sandbox-broker");
        assert_eq!(
            receipt.instance_id.as_deref(),
            Some("instance:sandbox:default")
        );
        assert_eq!(receipt.processing_epoch, Some(1));
        assert_eq!(
            receipt.lease_id.as_deref(),
            Some("lease:plugin-sandbox-broker")
        );
        assert_eq!(
            receipt.region_id.as_deref(),
            Some("region:plugin-sandbox-broker")
        );
        assert_eq!(receipt.detail, "lease_attached");
    }
}
