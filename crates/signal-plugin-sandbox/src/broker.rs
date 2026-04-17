use std::{
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

use signal_ipc::{PluginMessagePayload, SharedMemoryBroker};
use signal_plugin::PluginIoLayout;
use signal_plugin_au::{AuHostAdapter, AuHostPlatform};
use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_plugin_lv2::{
    Lv2HostAdapter, Lv2HostPlatform, Lv2InstanceControlSurface, Lv2ProcessSessionPlan,
    Lv2TeardownRecord,
};
use signal_plugin_vst3::{
    Vst3HostAdapter, Vst3HostPlatform, Vst3InstanceControlSurface, Vst3ProcessSessionPlan,
    Vst3StateSnapshot, Vst3TeardownRecord,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SandboxExecutionFlavor {
    Demo,
    Au,
    Lv2,
    Vst3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxBrokerState {
    Starting,
    Ready,
    Attached,
    Running,
    TeardownComplete,
    TimedOut,
    Crashed,
    Shutdown,
}

impl SandboxBrokerState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Attached => "attached",
            Self::Running => "running",
            Self::TeardownComplete => "teardown_complete",
            Self::TimedOut => "timed_out",
            Self::Crashed => "crashed",
            Self::Shutdown => "shutdown",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxBrokerReceipt {
    pub state: SandboxBrokerState,
    pub sandbox_id: String,
    pub instance_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub lease_id: Option<String>,
    pub region_id: Option<String>,
    pub detail: String,
}

impl SandboxBrokerReceipt {
    pub fn render_line(&self) -> String {
        format!(
            "signal-plugin-sandbox state={} sandbox_id={} instance_id={} epoch={} lease_id={} region_id={} detail={}",
            self.state.as_str(),
            self.sandbox_id,
            self.instance_id.as_deref().unwrap_or("-"),
            self.processing_epoch
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".into()),
            self.lease_id.as_deref().unwrap_or("-"),
            self.region_id.as_deref().unwrap_or("-"),
            self.detail.replace(' ', "_"),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SandboxBrokerCommand {
    Status,
    AttachDemo,
    AttachAu,
    AttachLv2,
    AttachVst3,
    ExecuteLv2,
    StreamLv2,
    StreamVst3,
    RefreshVst3,
    TimeoutVst3,
    RunDemo,
    RunAu,
    RunLv2,
    RunVst3,
    RunTimeoutDemo,
    RunTimeoutVst3,
    TeardownDemo,
    TeardownAu,
    TeardownLv2,
    TeardownVst3,
    Shutdown,
}

impl SandboxBrokerCommand {
    fn parse(line: &str) -> Result<Self, String> {
        match line.trim() {
            "status" => Ok(Self::Status),
            "attach-demo" => Ok(Self::AttachDemo),
            "attach-au" => Ok(Self::AttachAu),
            "attach-lv2" => Ok(Self::AttachLv2),
            "attach-vst3" => Ok(Self::AttachVst3),
            "execute-lv2" => Ok(Self::ExecuteLv2),
            "stream-lv2" => Ok(Self::StreamLv2),
            "stream-vst3" => Ok(Self::StreamVst3),
            "refresh-vst3" => Ok(Self::RefreshVst3),
            "timeout-vst3" => Ok(Self::TimeoutVst3),
            "run-demo" => Ok(Self::RunDemo),
            "run-au" => Ok(Self::RunAu),
            "run-lv2" => Ok(Self::RunLv2),
            "run-vst3" => Ok(Self::RunVst3),
            "run-timeout-demo" => Ok(Self::RunTimeoutDemo),
            "run-timeout-vst3" => Ok(Self::RunTimeoutVst3),
            "teardown-demo" => Ok(Self::TeardownDemo),
            "teardown-au" => Ok(Self::TeardownAu),
            "teardown-lv2" => Ok(Self::TeardownLv2),
            "teardown-vst3" => Ok(Self::TeardownVst3),
            "shutdown" => Ok(Self::Shutdown),
            other => Err(format!("unknown_command:{other}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SandboxRunSummary {
    sandbox_id: String,
    instance_id: String,
    processing_epoch: u64,
    lease_id: Option<String>,
    region_id: Option<String>,
    processed_blocks: usize,
    attached_detail: String,
    running_detail: String,
    teardown_detail: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Vst3BrokerExecution {
    instance_id: String,
    instance: Vst3InstanceControlSurface,
    session: Vst3ProcessSessionPlan,
    state: Vst3StateSnapshot,
    execution_runs: usize,
    teardown: Vst3TeardownRecord,
    attach_detail: String,
    teardown_detail: String,
}

#[derive(Clone, Debug, PartialEq)]
struct AuBrokerExecution {
    instance_id: String,
    attach_detail: String,
    teardown_detail: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Lv2BrokerExecution {
    instance_id: String,
    instance: Lv2InstanceControlSurface,
    session: Lv2ProcessSessionPlan,
    teardown: Lv2TeardownRecord,
    attach_detail: String,
    teardown_detail: String,
}

pub struct SandboxBrokerProcess {
    broker: SharedMemoryBroker,
    harness: ClapSandboxLifecycleHarness,
    protocol: ClapBlockProtocol,
    sandbox_id: String,
    last_summary: Option<SandboxRunSummary>,
    last_au_execution: Option<AuBrokerExecution>,
    last_lv2_execution: Option<Lv2BrokerExecution>,
    last_vst3_execution: Option<Vst3BrokerExecution>,
    last_state: SandboxBrokerState,
}

impl Default for SandboxBrokerProcess {
    fn default() -> Self {
        Self {
            broker: SharedMemoryBroker::default(),
            harness: ClapSandboxLifecycleHarness::default(),
            protocol: ClapBlockProtocol::new(
                "plugin:clap:sandbox",
                "instance:sandbox:default",
                PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 1,
                },
                2048,
            ),
            sandbox_id: "plugin-sandbox-broker".into(),
            last_summary: None,
            last_au_execution: None,
            last_lv2_execution: None,
            last_vst3_execution: None,
            last_state: SandboxBrokerState::Starting,
        }
    }
}

impl SandboxBrokerProcess {
    pub fn startup_receipts(&mut self) -> [SandboxBrokerReceipt; 2] {
        self.last_state = SandboxBrokerState::Ready;
        [
            SandboxBrokerReceipt {
                state: SandboxBrokerState::Starting,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: None,
                processing_epoch: None,
                lease_id: None,
                region_id: None,
                detail: "broker_boot".into(),
            },
            SandboxBrokerReceipt {
                state: SandboxBrokerState::Ready,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: None,
                processing_epoch: None,
                lease_id: None,
                region_id: None,
                detail: "awaiting_commands".into(),
            },
        ]
    }

    pub fn serve<R: BufRead, W: Write>(&mut self, input: R, mut output: W) -> io::Result<()> {
        for receipt in self.startup_receipts() {
            writeln!(output, "{}", receipt.render_line())?;
        }

        for line in input.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match SandboxBrokerCommand::parse(&line) {
                Ok(SandboxBrokerCommand::Status) => {
                    writeln!(output, "{}", self.status_receipt().render_line())?;
                }
                Ok(SandboxBrokerCommand::AttachDemo) => {
                    let receipt = self.attach(SandboxExecutionFlavor::Demo);
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::AttachAu) => {
                    let receipt = self.attach(SandboxExecutionFlavor::Au);
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::AttachLv2) => {
                    let receipt = self.attach(SandboxExecutionFlavor::Lv2);
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::AttachVst3) => {
                    let receipt = self.attach(SandboxExecutionFlavor::Vst3);
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::ExecuteLv2) => {
                    let receipt = self.execute_lv2();
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::StreamLv2) => {
                    for receipt in self.stream_lv2() {
                        writeln!(output, "{}", receipt.render_line())?;
                    }
                }
                Ok(SandboxBrokerCommand::StreamVst3) => {
                    for receipt in self.stream_vst3() {
                        writeln!(output, "{}", receipt.render_line())?;
                    }
                }
                Ok(SandboxBrokerCommand::RefreshVst3) => {
                    let receipt = self.refresh_vst3();
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::TimeoutVst3) => {
                    let receipt = self.timeout_vst3();
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::RunDemo) => {
                    for receipt in self.run(SandboxExecutionFlavor::Demo, false) {
                        writeln!(output, "{}", receipt.render_line())?;
                    }
                }
                Ok(SandboxBrokerCommand::RunAu) => {
                    for receipt in self.run(SandboxExecutionFlavor::Au, false) {
                        writeln!(output, "{}", receipt.render_line())?;
                    }
                }
                Ok(SandboxBrokerCommand::RunLv2) => {
                    for receipt in self.run(SandboxExecutionFlavor::Lv2, false) {
                        writeln!(output, "{}", receipt.render_line())?;
                    }
                }
                Ok(SandboxBrokerCommand::RunVst3) => {
                    for receipt in self.run(SandboxExecutionFlavor::Vst3, false) {
                        writeln!(output, "{}", receipt.render_line())?;
                    }
                }
                Ok(SandboxBrokerCommand::RunTimeoutDemo) => {
                    for receipt in self.run(SandboxExecutionFlavor::Demo, true) {
                        writeln!(output, "{}", receipt.render_line())?;
                    }
                }
                Ok(SandboxBrokerCommand::RunTimeoutVst3) => {
                    for receipt in self.run(SandboxExecutionFlavor::Vst3, true) {
                        writeln!(output, "{}", receipt.render_line())?;
                    }
                }
                Ok(SandboxBrokerCommand::TeardownDemo) => {
                    let receipt = self.teardown(SandboxExecutionFlavor::Demo);
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::TeardownAu) => {
                    let receipt = self.teardown(SandboxExecutionFlavor::Au);
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::TeardownLv2) => {
                    let receipt = self.teardown(SandboxExecutionFlavor::Lv2);
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::TeardownVst3) => {
                    let receipt = self.teardown(SandboxExecutionFlavor::Vst3);
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::Shutdown) => {
                    let receipt = self.shutdown_receipt();
                    writeln!(output, "{}", receipt.render_line())?;
                    return Ok(());
                }
                Err(error) => {
                    let receipt = SandboxBrokerReceipt {
                        state: SandboxBrokerState::Crashed,
                        sandbox_id: self.sandbox_id.clone(),
                        instance_id: self
                            .last_summary
                            .as_ref()
                            .map(|summary| summary.instance_id.clone()),
                        processing_epoch: self
                            .last_summary
                            .as_ref()
                            .map(|summary| summary.processing_epoch),
                        lease_id: self
                            .last_summary
                            .as_ref()
                            .and_then(|summary| summary.lease_id.clone()),
                        region_id: self
                            .last_summary
                            .as_ref()
                            .and_then(|summary| summary.region_id.clone()),
                        detail: error,
                    };
                    self.last_state = SandboxBrokerState::Crashed;
                    writeln!(output, "{}", receipt.render_line())?;
                }
            }
        }

        let receipt = self.shutdown_receipt();
        writeln!(output, "{}", receipt.render_line())?;
        Ok(())
    }

    fn status_receipt(&self) -> SandboxBrokerReceipt {
        SandboxBrokerReceipt {
            state: self.last_state,
            sandbox_id: self.sandbox_id.clone(),
            instance_id: self
                .last_summary
                .as_ref()
                .map(|summary| summary.instance_id.clone()),
            processing_epoch: self
                .last_summary
                .as_ref()
                .map(|summary| summary.processing_epoch),
            lease_id: self
                .last_summary
                .as_ref()
                .and_then(|summary| summary.lease_id.clone()),
            region_id: self
                .last_summary
                .as_ref()
                .and_then(|summary| summary.region_id.clone()),
            detail: "status".into(),
        }
    }

    fn shutdown_receipt(&mut self) -> SandboxBrokerReceipt {
        let shutdown_flavor = if self.last_vst3_execution.is_some() {
            SandboxExecutionFlavor::Vst3
        } else if self.last_lv2_execution.is_some() {
            SandboxExecutionFlavor::Lv2
        } else if self.last_au_execution.is_some() {
            SandboxExecutionFlavor::Au
        } else {
            SandboxExecutionFlavor::Demo
        };
        let _ = self.teardown(shutdown_flavor);
        self.last_state = SandboxBrokerState::Shutdown;
        SandboxBrokerReceipt {
            state: SandboxBrokerState::Shutdown,
            sandbox_id: self.sandbox_id.clone(),
            instance_id: self
                .last_summary
                .as_ref()
                .map(|summary| summary.instance_id.clone()),
            processing_epoch: self
                .last_summary
                .as_ref()
                .map(|summary| summary.processing_epoch),
            lease_id: None,
            region_id: None,
            detail: "broker_shutdown".into(),
        }
    }

    fn attach(&mut self, flavor: SandboxExecutionFlavor) -> SandboxBrokerReceipt {
        match self.execute_attach(flavor) {
            Ok(summary) => {
                self.last_state = SandboxBrokerState::Attached;
                self.last_summary = Some(summary.clone());
                SandboxBrokerReceipt {
                    state: SandboxBrokerState::Attached,
                    sandbox_id: summary.sandbox_id,
                    instance_id: Some(summary.instance_id),
                    processing_epoch: Some(summary.processing_epoch),
                    lease_id: summary.lease_id,
                    region_id: summary.region_id,
                    detail: summary.attached_detail,
                }
            }
            Err(error) => {
                self.last_state = SandboxBrokerState::Crashed;
                SandboxBrokerReceipt {
                    state: SandboxBrokerState::Crashed,
                    sandbox_id: self.sandbox_id.clone(),
                    instance_id: None,
                    processing_epoch: None,
                    lease_id: None,
                    region_id: None,
                    detail: error,
                }
            }
        }
    }

    fn run(
        &mut self,
        flavor: SandboxExecutionFlavor,
        simulate_timeout: bool,
    ) -> Vec<SandboxBrokerReceipt> {
        if flavor == SandboxExecutionFlavor::Vst3 && !simulate_timeout {
            return match self.execute_attach(flavor) {
                Ok(mut summary) => {
                    let stream = match self
                        .last_vst3_execution
                        .as_mut()
                        .ok_or_else(|| "missing_vst3_execution".to_string())
                        .and_then(Self::execute_vst3_block_stream)
                    {
                        Ok(stream) => stream,
                        Err(error) => {
                            self.last_state = SandboxBrokerState::Crashed;
                            return vec![SandboxBrokerReceipt {
                                state: SandboxBrokerState::Crashed,
                                sandbox_id: self.sandbox_id.clone(),
                                instance_id: None,
                                processing_epoch: None,
                                lease_id: None,
                                region_id: None,
                                detail: format!("vst3_execute_stream:{error}"),
                            }];
                        }
                    };
                    summary.processed_blocks = stream.len();
                    summary.running_detail = summarize_vst3_execution_stream(
                        &stream,
                        summary.processed_blocks,
                        self.last_vst3_execution
                            .as_ref()
                            .map(|execution| execution.execution_runs)
                            .unwrap_or(0),
                        None,
                    );
                    self.last_summary = Some(summary.clone());
                    let teardown_result = self.execute_teardown(flavor);
                    if let Err(error) = teardown_result {
                        self.last_state = SandboxBrokerState::Crashed;
                        return vec![SandboxBrokerReceipt {
                            state: SandboxBrokerState::Crashed,
                            sandbox_id: self.sandbox_id.clone(),
                            instance_id: Some(summary.instance_id.clone()),
                            processing_epoch: Some(summary.processing_epoch),
                            lease_id: summary.lease_id.clone(),
                            region_id: summary.region_id.clone(),
                            detail: error,
                        }];
                    }

                    self.last_state = SandboxBrokerState::TeardownComplete;
                    let mut receipts = vec![SandboxBrokerReceipt {
                        state: SandboxBrokerState::Attached,
                        sandbox_id: summary.sandbox_id.clone(),
                        instance_id: Some(summary.instance_id.clone()),
                        processing_epoch: Some(summary.processing_epoch),
                        lease_id: summary.lease_id.clone(),
                        region_id: summary.region_id.clone(),
                        detail: summary.attached_detail.clone(),
                    }];
                    receipts.extend(stream.iter().enumerate().map(|(index, record)| {
                        SandboxBrokerReceipt {
                            state: SandboxBrokerState::Running,
                            sandbox_id: summary.sandbox_id.clone(),
                            instance_id: Some(summary.instance_id.clone()),
                            processing_epoch: Some(summary.processing_epoch),
                            lease_id: summary.lease_id.clone(),
                            region_id: summary.region_id.clone(),
                            detail: format!(
                                "vst3:{}|stream_index={}|stream_complete={}",
                                record.summary,
                                index + 1,
                                index + 1 == stream.len()
                            ),
                        }
                    }));
                    receipts.push(SandboxBrokerReceipt {
                        state: SandboxBrokerState::TeardownComplete,
                        sandbox_id: summary.sandbox_id,
                        instance_id: Some(summary.instance_id),
                        processing_epoch: Some(summary.processing_epoch),
                        lease_id: None,
                        region_id: None,
                        detail: summary.teardown_detail,
                    });
                    receipts
                }
                Err(error) => {
                    self.last_state = SandboxBrokerState::Crashed;
                    vec![SandboxBrokerReceipt {
                        state: SandboxBrokerState::Crashed,
                        sandbox_id: self.sandbox_id.clone(),
                        instance_id: None,
                        processing_epoch: None,
                        lease_id: None,
                        region_id: None,
                        detail: error,
                    }]
                }
            };
        }

        match self.execute_run(flavor, simulate_timeout) {
            Ok(summary) => {
                self.last_summary = Some(summary.clone());
                let final_state = if simulate_timeout {
                    SandboxBrokerState::TimedOut
                } else {
                    SandboxBrokerState::TeardownComplete
                };
                self.last_state = final_state;
                vec![
                    SandboxBrokerReceipt {
                        state: SandboxBrokerState::Attached,
                        sandbox_id: summary.sandbox_id.clone(),
                        instance_id: Some(summary.instance_id.clone()),
                        processing_epoch: Some(summary.processing_epoch),
                        lease_id: summary.lease_id.clone(),
                        region_id: summary.region_id.clone(),
                        detail: summary.attached_detail.clone(),
                    },
                    SandboxBrokerReceipt {
                        state: if simulate_timeout {
                            SandboxBrokerState::TimedOut
                        } else {
                            SandboxBrokerState::Running
                        },
                        sandbox_id: summary.sandbox_id.clone(),
                        instance_id: Some(summary.instance_id.clone()),
                        processing_epoch: Some(summary.processing_epoch),
                        lease_id: summary.lease_id.clone(),
                        region_id: summary.region_id.clone(),
                        detail: if simulate_timeout {
                            timeout_detail_for(flavor)
                        } else {
                            summary.running_detail.clone()
                        },
                    },
                    SandboxBrokerReceipt {
                        state: final_state,
                        sandbox_id: summary.sandbox_id,
                        instance_id: Some(summary.instance_id),
                        processing_epoch: Some(summary.processing_epoch),
                        lease_id: None,
                        region_id: None,
                        detail: if simulate_timeout {
                            timeout_cleanup_detail_for(flavor)
                        } else {
                            summary.teardown_detail
                        },
                    },
                ]
            }
            Err(error) => {
                self.last_state = SandboxBrokerState::Crashed;
                vec![SandboxBrokerReceipt {
                    state: SandboxBrokerState::Crashed,
                    sandbox_id: self.sandbox_id.clone(),
                    instance_id: None,
                    processing_epoch: None,
                    lease_id: None,
                    region_id: None,
                    detail: error,
                }]
            }
        }
    }

    fn stream_vst3(&mut self) -> Vec<SandboxBrokerReceipt> {
        let Some(mut summary) = self.last_summary.clone() else {
            self.last_state = SandboxBrokerState::Crashed;
            return vec![SandboxBrokerReceipt {
                state: SandboxBrokerState::Crashed,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: None,
                processing_epoch: None,
                lease_id: None,
                region_id: None,
                detail: "missing_attached_vst3_session".into(),
            }];
        };

        let Some(execution) = self.last_vst3_execution.as_mut() else {
            self.last_state = SandboxBrokerState::Crashed;
            return vec![SandboxBrokerReceipt {
                state: SandboxBrokerState::Crashed,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: Some(summary.instance_id.clone()),
                processing_epoch: Some(summary.processing_epoch),
                lease_id: summary.lease_id.clone(),
                region_id: summary.region_id.clone(),
                detail: "missing_vst3_execution".into(),
            }];
        };

        let continuity_source = if execution.execution_runs == 0 {
            None
        } else {
            Some(execution.state.digest.clone())
        };

        let stream = match Self::execute_vst3_block_stream(execution) {
            Ok(stream) => stream,
            Err(error) => {
                self.last_state = SandboxBrokerState::Crashed;
                return vec![SandboxBrokerReceipt {
                    state: SandboxBrokerState::Crashed,
                    sandbox_id: self.sandbox_id.clone(),
                    instance_id: Some(summary.instance_id.clone()),
                    processing_epoch: Some(summary.processing_epoch),
                    lease_id: summary.lease_id.clone(),
                    region_id: summary.region_id.clone(),
                    detail: format!("vst3_stream:{error}"),
                }];
            }
        };

        summary.processed_blocks += stream.len();
        let final_detail = summarize_vst3_execution_stream(
            &stream,
            summary.processed_blocks,
            execution.execution_runs,
            continuity_source.as_deref(),
        );
        summary.running_detail = final_detail.clone();
        self.last_summary = Some(summary.clone());
        self.last_state = SandboxBrokerState::Attached;

        let mut receipts = stream
            .iter()
            .enumerate()
            .map(|(index, record)| SandboxBrokerReceipt {
                state: SandboxBrokerState::Running,
                sandbox_id: summary.sandbox_id.clone(),
                instance_id: Some(summary.instance_id.clone()),
                processing_epoch: Some(summary.processing_epoch),
                lease_id: summary.lease_id.clone(),
                region_id: summary.region_id.clone(),
                detail: format!(
                    "vst3:{}|stream_index={}|stream_complete={}",
                    record.summary,
                    index + 1,
                    index + 1 == stream.len()
                ),
            })
            .collect::<Vec<_>>();
        receipts.push(SandboxBrokerReceipt {
            state: SandboxBrokerState::Attached,
            sandbox_id: summary.sandbox_id,
            instance_id: Some(summary.instance_id),
            processing_epoch: Some(summary.processing_epoch),
            lease_id: summary.lease_id,
            region_id: summary.region_id,
            detail: final_detail,
        });
        receipts
    }

    fn teardown(&mut self, flavor: SandboxExecutionFlavor) -> SandboxBrokerReceipt {
        let Some(summary) = self.last_summary.clone() else {
            return SandboxBrokerReceipt {
                state: SandboxBrokerState::TeardownComplete,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: None,
                processing_epoch: None,
                lease_id: None,
                region_id: None,
                detail: teardown_noop_detail_for(flavor),
            };
        };

        let detail = match self.execute_teardown(flavor) {
            Ok(()) => summary.teardown_detail.clone(),
            Err(error) => {
                self.last_state = SandboxBrokerState::Crashed;
                return SandboxBrokerReceipt {
                    state: SandboxBrokerState::Crashed,
                    sandbox_id: summary.sandbox_id,
                    instance_id: Some(summary.instance_id),
                    processing_epoch: Some(summary.processing_epoch),
                    lease_id: summary.lease_id,
                    region_id: summary.region_id,
                    detail: error,
                };
            }
        };

        self.last_state = SandboxBrokerState::TeardownComplete;
        self.last_summary = None;
        SandboxBrokerReceipt {
            state: SandboxBrokerState::TeardownComplete,
            sandbox_id: summary.sandbox_id,
            instance_id: Some(summary.instance_id),
            processing_epoch: Some(summary.processing_epoch),
            lease_id: None,
            region_id: None,
            detail,
        }
    }

    fn refresh_vst3(&mut self) -> SandboxBrokerReceipt {
        let Some(mut summary) = self.last_summary.clone() else {
            self.last_state = SandboxBrokerState::Crashed;
            return SandboxBrokerReceipt {
                state: SandboxBrokerState::Crashed,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: None,
                processing_epoch: None,
                lease_id: None,
                region_id: None,
                detail: "missing_attached_vst3_session".into(),
            };
        };

        let Some(execution) = self.last_vst3_execution.as_mut() else {
            self.last_state = SandboxBrokerState::Crashed;
            return SandboxBrokerReceipt {
                state: SandboxBrokerState::Crashed,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: Some(summary.instance_id.clone()),
                processing_epoch: Some(summary.processing_epoch),
                lease_id: summary.lease_id.clone(),
                region_id: summary.region_id.clone(),
                detail: "missing_vst3_execution".into(),
            };
        };

        let previous_state = execution.state.digest.clone();
        match Self::refresh_vst3_state(execution) {
            Ok(detail) => {
                summary.processed_blocks = 0;
                summary.running_detail = detail.clone();
                self.last_summary = Some(summary.clone());
                self.last_state = SandboxBrokerState::Attached;
                SandboxBrokerReceipt {
                    state: SandboxBrokerState::Attached,
                    sandbox_id: summary.sandbox_id,
                    instance_id: Some(summary.instance_id),
                    processing_epoch: Some(summary.processing_epoch),
                    lease_id: summary.lease_id,
                    region_id: summary.region_id,
                    detail: format!("{detail}|previous_state={previous_state}"),
                }
            }
            Err(error) => {
                self.last_state = SandboxBrokerState::Crashed;
                SandboxBrokerReceipt {
                    state: SandboxBrokerState::Crashed,
                    sandbox_id: self.sandbox_id.clone(),
                    instance_id: Some(summary.instance_id),
                    processing_epoch: Some(summary.processing_epoch),
                    lease_id: summary.lease_id,
                    region_id: summary.region_id,
                    detail: format!("vst3_refresh:{error}"),
                }
            }
        }
    }

    fn execute_lv2(&mut self) -> SandboxBrokerReceipt {
        let Some(mut summary) = self.last_summary.clone() else {
            self.last_state = SandboxBrokerState::Crashed;
            return SandboxBrokerReceipt {
                state: SandboxBrokerState::Crashed,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: None,
                processing_epoch: None,
                lease_id: None,
                region_id: None,
                detail: "missing_attached_lv2_session".into(),
            };
        };

        let Some(execution) = self.last_lv2_execution.as_ref() else {
            self.last_state = SandboxBrokerState::Crashed;
            return SandboxBrokerReceipt {
                state: SandboxBrokerState::Crashed,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: Some(summary.instance_id.clone()),
                processing_epoch: Some(summary.processing_epoch),
                lease_id: summary.lease_id.clone(),
                region_id: summary.region_id.clone(),
                detail: "missing_lv2_execution".into(),
            };
        };

        let record = Lv2HostAdapter::default().execute_block(
            &execution.instance,
            &execution.session,
            0,
            256,
            2,
            1,
        );
        summary.processed_blocks = 1;
        summary.running_detail = format!(
            "execution_complete|processed_blocks=1|block_sequence={}|block_frames={}|audio_outputs={}|midi_events={}|patch_messages={}|completion={}|{}",
            record.block_sequence,
            record.block_frames,
            record.audio_output_channels,
            record.midi_event_count,
            record.patch_message_count,
            record.completion_status,
            record.summary,
        );
        self.last_summary = Some(summary.clone());
        self.last_state = SandboxBrokerState::Attached;

        SandboxBrokerReceipt {
            state: SandboxBrokerState::Attached,
            sandbox_id: summary.sandbox_id,
            instance_id: Some(summary.instance_id),
            processing_epoch: Some(summary.processing_epoch),
            lease_id: summary.lease_id,
            region_id: summary.region_id,
            detail: summary.running_detail,
        }
    }

    fn stream_lv2(&mut self) -> Vec<SandboxBrokerReceipt> {
        let Some(mut summary) = self.last_summary.clone() else {
            self.last_state = SandboxBrokerState::Crashed;
            return vec![SandboxBrokerReceipt {
                state: SandboxBrokerState::Crashed,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: None,
                processing_epoch: None,
                lease_id: None,
                region_id: None,
                detail: "missing_attached_lv2_session".into(),
            }];
        };

        let Some(execution) = self.last_lv2_execution.as_ref() else {
            self.last_state = SandboxBrokerState::Crashed;
            return vec![SandboxBrokerReceipt {
                state: SandboxBrokerState::Crashed,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: Some(summary.instance_id.clone()),
                processing_epoch: Some(summary.processing_epoch),
                lease_id: summary.lease_id.clone(),
                region_id: summary.region_id.clone(),
                detail: "missing_lv2_execution".into(),
            }];
        };

        let stream = Self::execute_lv2_block_stream(execution);
        summary.processed_blocks = stream.len();
        let final_detail = summarize_lv2_execution_stream(&stream);
        summary.running_detail = final_detail.clone();
        self.last_summary = Some(summary.clone());
        self.last_state = SandboxBrokerState::Attached;

        let mut receipts = stream
            .iter()
            .enumerate()
            .map(|(index, record)| SandboxBrokerReceipt {
                state: SandboxBrokerState::Running,
                sandbox_id: summary.sandbox_id.clone(),
                instance_id: Some(summary.instance_id.clone()),
                processing_epoch: Some(summary.processing_epoch),
                lease_id: summary.lease_id.clone(),
                region_id: summary.region_id.clone(),
                detail: format!(
                    "lv2:{}|stream_index={}|stream_complete={}",
                    record.summary,
                    index + 1,
                    index + 1 == stream.len()
                ),
            })
            .collect::<Vec<_>>();
        receipts.push(SandboxBrokerReceipt {
            state: SandboxBrokerState::Attached,
            sandbox_id: summary.sandbox_id,
            instance_id: Some(summary.instance_id),
            processing_epoch: Some(summary.processing_epoch),
            lease_id: summary.lease_id,
            region_id: summary.region_id,
            detail: final_detail,
        });
        receipts
    }

    fn timeout_vst3(&mut self) -> SandboxBrokerReceipt {
        let Some(summary) = self.last_summary.clone() else {
            self.last_state = SandboxBrokerState::Crashed;
            return SandboxBrokerReceipt {
                state: SandboxBrokerState::Crashed,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: None,
                processing_epoch: None,
                lease_id: None,
                region_id: None,
                detail: "missing_attached_vst3_session".into(),
            };
        };

        let Some(execution) = self.last_vst3_execution.as_ref() else {
            self.last_state = SandboxBrokerState::Crashed;
            return SandboxBrokerReceipt {
                state: SandboxBrokerState::Crashed,
                sandbox_id: self.sandbox_id.clone(),
                instance_id: Some(summary.instance_id),
                processing_epoch: Some(summary.processing_epoch),
                lease_id: summary.lease_id,
                region_id: summary.region_id,
                detail: "missing_vst3_execution".into(),
            };
        };

        self.last_state = SandboxBrokerState::Attached;
        SandboxBrokerReceipt {
            state: SandboxBrokerState::Attached,
            sandbox_id: summary.sandbox_id,
            instance_id: Some(summary.instance_id),
            processing_epoch: Some(summary.processing_epoch),
            lease_id: summary.lease_id,
            region_id: summary.region_id,
            detail: format!(
                "execution_interrupted|timeout=recoverable|continuity=carried_forward|execution_runs={}|state_digest={}|resume_hint=refresh_or_stream",
                execution.execution_runs,
                execution.state.digest
            ),
        }
    }

    fn execute_attach(
        &mut self,
        flavor: SandboxExecutionFlavor,
    ) -> Result<SandboxRunSummary, String> {
        if self.last_summary.is_some() {
            return Err("already_attached".into());
        }
        if flavor == SandboxExecutionFlavor::Au {
            let execution = self.prepare_au_execution()?;
            self.last_au_execution = Some(execution.clone());
            return Ok(SandboxRunSummary {
                sandbox_id: self.sandbox_id.clone(),
                instance_id: execution.instance_id.clone(),
                processing_epoch: 1,
                lease_id: Some(format!("lease:{}", self.sandbox_id)),
                region_id: Some(format!("region:{}", self.sandbox_id)),
                processed_blocks: 0,
                attached_detail: execution.attach_detail,
                running_detail: "au:lifecycle_attached".into(),
                teardown_detail: execution.teardown_detail,
            });
        }
        if flavor == SandboxExecutionFlavor::Lv2 {
            let execution = self.prepare_lv2_execution()?;
            self.last_lv2_execution = Some(execution.clone());
            return Ok(SandboxRunSummary {
                sandbox_id: self.sandbox_id.clone(),
                instance_id: execution.instance_id.clone(),
                processing_epoch: 1,
                lease_id: Some(format!("lease:{}", self.sandbox_id)),
                region_id: Some(format!("region:{}", self.sandbox_id)),
                processed_blocks: 0,
                attached_detail: execution.attach_detail,
                running_detail: "lv2:lifecycle_attached".into(),
                teardown_detail: execution.teardown_detail,
            });
        }
        if flavor == SandboxExecutionFlavor::Vst3 {
            let execution = self.prepare_vst3_execution()?;
            self.last_vst3_execution = Some(execution.clone());
            return Ok(SandboxRunSummary {
                sandbox_id: self.sandbox_id.clone(),
                instance_id: execution.instance_id.clone(),
                processing_epoch: 1,
                lease_id: Some(format!("lease:{}", self.sandbox_id)),
                region_id: Some(format!("region:{}", self.sandbox_id)),
                processed_blocks: 0,
                attached_detail: execution.attach_detail,
                running_detail: "vst3:execution_pending".into(),
                teardown_detail: execution.teardown_detail,
            });
        }
        self.protocol = ClapBlockProtocol::new(
            "plugin:clap:sandbox",
            "instance:sandbox:default",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            2048,
        );
        self.harness = ClapSandboxLifecycleHarness::default();
        let requests = self
            .protocol
            .lifecycle_sequence(&self.broker, &self.sandbox_id, 48_000, 512, 1)
            .map_err(|error| format!("lifecycle_sequence:{error}"))?;

        let responses = requests
            .iter()
            .cloned()
            .map(|request| self.harness.handle(request))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("lifecycle:{error:?}"))?;

        let (lease_id, region_id) = responses
            .iter()
            .find_map(|response| match &response.payload {
                PluginMessagePayload::PrepareInstanceResponse {
                    shared_memory_lease_id,
                    shared_memory_transport,
                    ..
                } => Some((
                    Some(shared_memory_lease_id.clone()),
                    Some(shared_memory_transport.region_id.clone()),
                )),
                _ => None,
            })
            .ok_or_else(|| "missing_prepare_transport".to_string())?;

        let transport_instance_id = responses
            .iter()
            .find_map(|response| match &response.payload {
                PluginMessagePayload::CreateInstanceResponse { instance_id, .. } => {
                    Some(instance_id.clone())
                }
                _ => None,
            })
            .ok_or_else(|| "missing_instance_id".to_string())?;
        let instance_id = transport_instance_id;

        let processing_epoch = responses
            .iter()
            .find_map(|response| match &response.payload {
                PluginMessagePayload::PrepareInstanceResponse {
                    processing_epoch, ..
                } => Some(*processing_epoch),
                _ => None,
            })
            .ok_or_else(|| "missing_processing_epoch".to_string())?;

        let heartbeat = self
            .harness
            .handle(
                self.protocol
                    .heartbeat_request(&self.sandbox_id, Some(processing_epoch)),
            )
            .map_err(|error| format!("heartbeat:{error:?}"))?;
        if !matches!(
            heartbeat.payload,
            PluginMessagePayload::HeartbeatResponse { .. }
        ) {
            return Err("heartbeat_response_missing".into());
        }

        Ok(SandboxRunSummary {
            sandbox_id: self.sandbox_id.clone(),
            instance_id,
            processing_epoch,
            lease_id,
            region_id,
            processed_blocks: 0,
            attached_detail: attached_detail_for(flavor),
            running_detail: running_detail_for(flavor, 0),
            teardown_detail: teardown_detail_for(flavor),
        })
    }

    fn execute_run(
        &mut self,
        flavor: SandboxExecutionFlavor,
        simulate_timeout: bool,
    ) -> Result<SandboxRunSummary, String> {
        let mut summary = self.execute_attach(flavor)?;
        if flavor == SandboxExecutionFlavor::Au {
            summary.processed_blocks = 0;
            summary.running_detail = if simulate_timeout {
                timeout_detail_for(flavor)
            } else {
                running_detail_for(flavor, 0)
            };
            self.execute_teardown(flavor)?;
            return Ok(summary);
        }
        if flavor == SandboxExecutionFlavor::Lv2 {
            if simulate_timeout {
                summary.processed_blocks = 0;
                summary.running_detail = timeout_detail_for(flavor);
            } else {
                let execution = self
                    .last_lv2_execution
                    .as_ref()
                    .ok_or_else(|| "missing_lv2_execution".to_string())?;
                let stream = Self::execute_lv2_block_stream(execution);
                summary.processed_blocks = stream.len();
                summary.running_detail = summarize_lv2_execution_stream(&stream);
            }
            self.execute_teardown(flavor)?;
            return Ok(summary);
        }
        if flavor == SandboxExecutionFlavor::Vst3 {
            let processed_blocks = if simulate_timeout { 0 } else { 8 };
            summary.processed_blocks = processed_blocks;
            if simulate_timeout {
                summary.running_detail = timeout_detail_for(flavor);
            } else {
                let execution = self
                    .last_vst3_execution
                    .as_mut()
                    .ok_or_else(|| "missing_vst3_execution".to_string())?;
                let stream = Self::execute_vst3_block_stream(execution)
                    .map_err(|error| format!("vst3_execute_stream:{error}"))?;
                summary.processed_blocks = stream.len();
                summary.running_detail = summarize_vst3_execution_stream(
                    &stream,
                    summary.processed_blocks,
                    execution.execution_runs,
                    None,
                );
            }
            self.execute_teardown(flavor)?;
            return Ok(summary);
        }
        let transport = self
            .harness
            .lease()
            .and_then(|lease| lease.transport().cloned())
            .ok_or_else(|| "missing_active_transport".to_string())?;

        let processed_blocks = if simulate_timeout {
            0
        } else {
            for block_sequence in 0..8 {
                let dispatch = self.protocol.block_dispatch(
                    summary.processing_epoch,
                    block_sequence,
                    512,
                    self.protocol.default_render_context(512),
                );
                let payload = self.protocol.test_input_payload(block_sequence, 512);
                self.protocol
                    .write_block_payload(&self.broker, &transport, &dispatch, &payload)
                    .map_err(|error| format!("write_block:{error}"))?;
                self.harness
                    .process_pending_block()
                    .map_err(|error| format!("process_block:{error:?}"))?;
                let _ = self
                    .protocol
                    .read_block_outcome(&self.broker, &transport, &dispatch)
                    .map_err(|error| format!("read_block:{error}"))?;
            }
            8
        };
        self.execute_teardown(flavor)?;
        summary.processed_blocks = processed_blocks;
        summary.running_detail = running_detail_for(flavor, processed_blocks);
        Ok(summary)
    }

    fn execute_teardown(&mut self, flavor: SandboxExecutionFlavor) -> Result<(), String> {
        if flavor == SandboxExecutionFlavor::Au {
            self.last_au_execution = None;
            return Ok(());
        }
        if flavor == SandboxExecutionFlavor::Lv2 {
            self.last_lv2_execution = None;
            return Ok(());
        }
        if flavor == SandboxExecutionFlavor::Vst3 {
            self.last_vst3_execution = None;
            return Ok(());
        }
        let Some(summary) = self.last_summary.as_ref() else {
            return Ok(());
        };

        for request in self
            .protocol
            .teardown_sequence(&self.sandbox_id, summary.processing_epoch)
        {
            self.harness
                .handle(request)
                .map_err(|error| format!("teardown:{error:?}"))?;
        }

        self.harness
            .teardown_active_transport()
            .map_err(|error| format!("teardown_transport:{error}"))?;
        self.last_vst3_execution = None;
        Ok(())
    }

    fn prepare_au_execution(&self) -> Result<AuBrokerExecution, String> {
        let plugin_type_id = std::env::var("SIGNAL_PLUGIN_SANDBOX_AU_PLUGIN_TYPE_ID")
            .map_err(|_| "missing_au_plugin_type_id".to_string())?;
        let bundle_root = std::env::var("SIGNAL_PLUGIN_SANDBOX_AU_BUNDLE_ROOT")
            .map_err(|_| "missing_au_bundle_root".to_string())?;
        let instance_id = std::env::var("SIGNAL_PLUGIN_SANDBOX_AU_INSTANCE_ID")
            .unwrap_or_else(|_| "instance:sandbox:au".into());

        let bundle_root_path = PathBuf::from(&bundle_root);
        let scan_root = bundle_root_path
            .parent()
            .ok_or_else(|| "invalid_au_bundle_root".to_string())?;
        let adapter = AuHostAdapter::default();
        let discovered = adapter
            .discover_plugins_for_roots(AuHostPlatform::MacOs, &[scan_root.display().to_string()])
            .into_iter()
            .find(|plugin| {
                plugin.plugin_type_id.0 == plugin_type_id
                    && Path::new(&plugin.bundle_root) == bundle_root_path
            })
            .ok_or_else(|| "au_broker_discovery_miss".to_string())?;
        let instance = adapter
            .instantiate_plugin(&discovered, &instance_id)
            .map_err(|error| format!("au_instantiate:{error}"))?;
        let state = adapter.store_state_snapshot(&instance);
        let activation = adapter
            .activate_instance(&instance, 48_000, 512, Some(&state))
            .map_err(|error| format!("au_activate:{error}"))?;
        let teardown = adapter.teardown_instance(&instance, Some(&state));
        let attach_detail = format!("lease_attached|au:{}|{}", state.summary, activation.summary);
        let teardown_detail = format!("lease_cleanup_ok|au:{}", teardown.summary);

        Ok(AuBrokerExecution {
            instance_id,
            attach_detail,
            teardown_detail,
        })
    }

    fn prepare_lv2_execution(&self) -> Result<Lv2BrokerExecution, String> {
        let plugin_type_id = std::env::var("SIGNAL_PLUGIN_SANDBOX_LV2_PLUGIN_TYPE_ID")
            .map_err(|_| "missing_lv2_plugin_type_id".to_string())?;
        let bundle_root = std::env::var("SIGNAL_PLUGIN_SANDBOX_LV2_BUNDLE_ROOT")
            .map_err(|_| "missing_lv2_bundle_root".to_string())?;
        let instance_id = std::env::var("SIGNAL_PLUGIN_SANDBOX_LV2_INSTANCE_ID")
            .unwrap_or_else(|_| "instance:sandbox:lv2".into());

        let bundle_root_path = PathBuf::from(&bundle_root);
        let scan_root = bundle_root_path
            .parent()
            .ok_or_else(|| "invalid_lv2_bundle_root".to_string())?;
        let adapter = Lv2HostAdapter::default();
        let discovered = adapter
            .discover_plugins_for_roots(Lv2HostPlatform::Linux, &[scan_root.display().to_string()])
            .into_iter()
            .find(|plugin| {
                plugin.plugin_type_id.0 == plugin_type_id
                    && Path::new(&plugin.bundle_root) == bundle_root_path
            })
            .ok_or_else(|| "lv2_broker_discovery_miss".to_string())?;
        let instance = adapter.instantiate_plugin(&discovered, &instance_id);
        let session = adapter.prepare_session(&instance, 48_000, 512);
        let teardown = adapter.teardown_instance(&instance, &session);
        let attach_detail = format!("lease_attached|lv2:{}", session.summary);
        let teardown_detail = format!("lease_cleanup_ok|lv2:{}", teardown.summary);

        Ok(Lv2BrokerExecution {
            instance_id,
            instance,
            session,
            teardown,
            attach_detail,
            teardown_detail,
        })
    }

    fn prepare_vst3_execution(&self) -> Result<Vst3BrokerExecution, String> {
        let plugin_type_id = std::env::var("SIGNAL_PLUGIN_SANDBOX_VST3_PLUGIN_TYPE_ID")
            .map_err(|_| "missing_vst3_plugin_type_id".to_string())?;
        let module_root = std::env::var("SIGNAL_PLUGIN_SANDBOX_VST3_MODULE_ROOT")
            .map_err(|_| "missing_vst3_module_root".to_string())?;
        let instance_id = std::env::var("SIGNAL_PLUGIN_SANDBOX_VST3_INSTANCE_ID")
            .unwrap_or_else(|_| "instance:sandbox:vst3".into());

        let module_root_path = PathBuf::from(&module_root);
        let scan_root = module_root_path
            .parent()
            .ok_or_else(|| "invalid_vst3_module_root".to_string())?;
        let adapter = Vst3HostAdapter::default();
        let discovered = adapter
            .discover_plugins_for_roots(current_vst3_platform(), &[scan_root.display().to_string()])
            .into_iter()
            .find(|plugin| {
                plugin.plugin_type_id.0 == plugin_type_id
                    && Path::new(&plugin.module_root) == module_root_path
            })
            .ok_or_else(|| "vst3_broker_discovery_miss".to_string())?;
        let instance = adapter
            .instantiate_plugin(&discovered, &instance_id)
            .map_err(|error| format!("vst3_instantiate:{error}"))?;
        let state = adapter
            .store_state_snapshot(&instance)
            .map_err(|error| format!("vst3_store_state:{error}"))?;
        let activation = adapter
            .activate_instance(&instance, 48_000, 512, Some(&state))
            .map_err(|error| format!("vst3_activate:{error}"))?;
        let session = adapter.prepare_session(&instance, 48_000, 512);
        let teardown = adapter
            .teardown_instance(&instance, Some(&state))
            .map_err(|error| format!("vst3_teardown:{error}"))?;
        let attach_detail = format!(
            "lease_attached|vst3:{}|{}",
            state.summary, activation.summary
        );
        let teardown_detail = format!("lease_cleanup_ok|vst3:{}", teardown.summary);

        Ok(Vst3BrokerExecution {
            instance_id,
            instance,
            session,
            state,
            execution_runs: 0,
            teardown: teardown.clone(),
            attach_detail,
            teardown_detail,
        })
    }

    fn execute_vst3_block_run(
        execution: &Vst3BrokerExecution,
        state: &Vst3StateSnapshot,
        block_sequence: u64,
        block_frames: u32,
        parameter_event_count: u16,
        midi_event_count: u16,
    ) -> Result<signal_plugin_vst3::Vst3BlockProcessingRecord, String> {
        let adapter = Vst3HostAdapter::default();
        adapter
            .execute_block(
                &execution.instance,
                &execution.session,
                Some(state),
                block_sequence,
                block_frames,
                parameter_event_count,
                midi_event_count,
            )
            .map_err(|error| error.to_string())
    }

    fn execute_vst3_block_stream(
        execution: &mut Vst3BrokerExecution,
    ) -> Result<Vec<signal_plugin_vst3::Vst3BlockProcessingRecord>, String> {
        const VST3_STREAM_BLOCKS: &[(u32, u16, u16)] = &[(128, 2, 1), (192, 4, 0), (256, 1, 2)];
        let mut stream = Vec::with_capacity(VST3_STREAM_BLOCKS.len());
        let mut state = execution.state.clone();
        let run_delta = execution.execution_runs as u16;

        for (index, (block_frames, parameter_event_count, midi_event_count)) in
            VST3_STREAM_BLOCKS.iter().enumerate()
        {
            let record = Self::execute_vst3_block_run(
                execution,
                &state,
                index as u64,
                *block_frames,
                parameter_event_count.saturating_add(run_delta),
                midi_event_count.saturating_add(run_delta),
            )?;
            state = Vst3StateSnapshot {
                instance_id: state.instance_id.clone(),
                bytes: format!(
                    "previous_state={} next_state={} parameter_signature={}",
                    state.digest, record.next_state_digest, record.parameter_signature
                )
                .into_bytes(),
                digest: record.next_state_digest.clone(),
                summary: format!(
                    "instance={} streamed_state_digest={} parameter_signature={}",
                    state.instance_id.0, record.next_state_digest, record.parameter_signature
                ),
            };
            stream.push(record);
        }

        execution.state = state;
        execution.execution_runs += 1;

        Ok(stream)
    }

    fn execute_lv2_block_stream(
        execution: &Lv2BrokerExecution,
    ) -> Vec<signal_plugin_lv2::Lv2BlockProcessingRecord> {
        const LV2_STREAM_BLOCKS: &[(u32, u16, u16)] = &[(128, 1, 0), (192, 2, 1), (256, 1, 2)];
        let adapter = Lv2HostAdapter::default();
        LV2_STREAM_BLOCKS
            .iter()
            .enumerate()
            .map(|(index, (block_frames, patch_messages, midi_events))| {
                adapter.execute_block(
                    &execution.instance,
                    &execution.session,
                    index as u64,
                    *block_frames,
                    *patch_messages,
                    *midi_events,
                )
            })
            .collect()
    }

    fn refresh_vst3_state(execution: &mut Vst3BrokerExecution) -> Result<String, String> {
        let adapter = Vst3HostAdapter::default();
        let refreshed_state = adapter
            .store_state_snapshot(&execution.instance)
            .map_err(|error| format!("vst3_store_state_refresh:{error}"))?;
        let refresh_summary = format!(
            "refresh_cycle=state_store|continuity_reset=refreshed|refreshed_state={}|{}",
            refreshed_state.digest, refreshed_state.summary
        );
        execution.state = refreshed_state;
        execution.execution_runs = 0;
        Ok(refresh_summary)
    }
}

fn current_vst3_platform() -> Vst3HostPlatform {
    #[cfg(target_os = "macos")]
    {
        Vst3HostPlatform::MacOs
    }
    #[cfg(target_os = "windows")]
    {
        Vst3HostPlatform::Windows
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        Vst3HostPlatform::Linux
    }
}

fn attached_detail_for(flavor: SandboxExecutionFlavor) -> String {
    match flavor {
        SandboxExecutionFlavor::Demo => "lease_attached".into(),
        SandboxExecutionFlavor::Au => "lease_attached|au:state_stored=1|activation=ready".into(),
        SandboxExecutionFlavor::Lv2 => {
            "lease_attached|lv2:prepared_negotiation=ready|transport=shared_memory".into()
        }
        SandboxExecutionFlavor::Vst3 => {
            "lease_attached|vst3:state_stored=1|state_bytes=192|activation=ready".into()
        }
    }
}

fn running_detail_for(flavor: SandboxExecutionFlavor, processed_blocks: usize) -> String {
    match flavor {
        SandboxExecutionFlavor::Demo => format!("processed_blocks={processed_blocks}"),
        SandboxExecutionFlavor::Au => {
            "processed_blocks=0|au:lifecycle_ready|state_snapshot=stored".into()
        }
        SandboxExecutionFlavor::Lv2 => {
            "processed_blocks=0|lv2:lifecycle_ready|prepared_negotiation=recorded".into()
        }
        SandboxExecutionFlavor::Vst3 => format!(
            "processed_blocks={processed_blocks}|vst3:activated_sr=48000|max_block_frames=512"
        ),
    }
}

fn summarize_vst3_execution_stream(
    stream: &[signal_plugin_vst3::Vst3BlockProcessingRecord],
    processed_blocks: usize,
    execution_runs: usize,
    continuity_source: Option<&str>,
) -> String {
    let last = stream
        .last()
        .expect("VST3 execution stream summary requires at least one block");
    let application_order = stream
        .iter()
        .map(|record| record.parameter_application_order.as_str())
        .collect::<Vec<_>>()
        .join(";");
    let packet_order = stream
        .iter()
        .map(|record| record.event_packet_order.as_str())
        .collect::<Vec<_>>()
        .join(";");
    let automation_delta = stream
        .iter()
        .map(|record| record.automation_delta.as_str())
        .collect::<Vec<_>>()
        .join(";");
    let continuity = if continuity_source.is_some() {
        "carried_forward"
    } else {
        "fresh"
    };
    format!(
        "execution_complete|processed_blocks={processed_blocks}|execution_runs={execution_runs}|continuity={continuity}|continued_from={}|last_block_sequence={}|last_block_frames={}|audio_outputs={}|parameter_events={}|midi_events={}|completion={}|state_digest={}|parameter_signature={}|application_order={}|packet_order={}|automation_delta={}|next_state_digest={}|{}",
        continuity_source.unwrap_or("none"),
        last.block_sequence,
        last.block_frames,
        last.audio_output_channels,
        last.parameter_event_count,
        last.midi_event_count,
        last.completion_status,
        last.state_digest.as_deref().unwrap_or("none"),
        last.parameter_signature,
        application_order,
        packet_order,
        automation_delta,
        last.next_state_digest,
        last.state_transition,
    )
}

fn summarize_lv2_execution_stream(
    stream: &[signal_plugin_lv2::Lv2BlockProcessingRecord],
) -> String {
    let last = stream
        .last()
        .expect("LV2 execution stream summary requires at least one block");
    let stream_order = stream
        .iter()
        .map(|record| {
            format!(
                "block{}[frames={},patch={},midi={},completion={}]",
                record.block_sequence,
                record.block_frames,
                record.patch_message_count,
                record.midi_event_count,
                record.completion_status
            )
        })
        .collect::<Vec<_>>()
        .join(";");
    format!(
        "execution_complete|processed_blocks={}|last_block_sequence={}|last_block_frames={}|audio_outputs={}|patch_messages={}|midi_events={}|completion={}|stream_order={}|{}",
        stream.len(),
        last.block_sequence,
        last.block_frames,
        last.audio_output_channels,
        last.patch_message_count,
        last.midi_event_count,
        last.completion_status,
        stream_order,
        last.summary,
    )
}

fn teardown_detail_for(flavor: SandboxExecutionFlavor) -> String {
    match flavor {
        SandboxExecutionFlavor::Demo => "lease_cleanup_ok".into(),
        SandboxExecutionFlavor::Au => {
            "lease_cleanup_ok|au:flushed_state_bytes=160|suspended=1".into()
        }
        SandboxExecutionFlavor::Lv2 => "lease_cleanup_ok|lv2:prepared_negotiation_flushed=1".into(),
        SandboxExecutionFlavor::Vst3 => {
            "lease_cleanup_ok|vst3:flushed_state_bytes=192|suspended=1".into()
        }
    }
}

fn timeout_detail_for(flavor: SandboxExecutionFlavor) -> String {
    match flavor {
        SandboxExecutionFlavor::Demo => "lease_attached_block_processing_timeout".into(),
        SandboxExecutionFlavor::Au => {
            "lease_attached_block_processing_timeout|au:activation=ready".into()
        }
        SandboxExecutionFlavor::Lv2 => {
            "lease_attached_block_processing_timeout|lv2:prepared_negotiation=ready".into()
        }
        SandboxExecutionFlavor::Vst3 => {
            "lease_attached_block_processing_timeout|vst3:activated_sr=48000".into()
        }
    }
}

fn timeout_cleanup_detail_for(flavor: SandboxExecutionFlavor) -> String {
    match flavor {
        SandboxExecutionFlavor::Demo => "lease_cleanup_ok_after_timeout".into(),
        SandboxExecutionFlavor::Au => {
            "lease_cleanup_ok_after_timeout|au:flushed_state_bytes=160".into()
        }
        SandboxExecutionFlavor::Lv2 => {
            "lease_cleanup_ok_after_timeout|lv2:prepared_negotiation_flushed=1".into()
        }
        SandboxExecutionFlavor::Vst3 => {
            "lease_cleanup_ok_after_timeout|vst3:flushed_state_bytes=192".into()
        }
    }
}

fn teardown_noop_detail_for(flavor: SandboxExecutionFlavor) -> String {
    match flavor {
        SandboxExecutionFlavor::Demo => "lease_cleanup_noop".into(),
        SandboxExecutionFlavor::Au => "lease_cleanup_noop|au:flushed_state_bytes=0".into(),
        SandboxExecutionFlavor::Lv2 => {
            "lease_cleanup_noop|lv2:prepared_negotiation_flushed=0".into()
        }
        SandboxExecutionFlavor::Vst3 => "lease_cleanup_noop|vst3:flushed_state_bytes=0".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        io::Cursor,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Vst3EnvGuard {
        old_plugin_type_id: Option<std::ffi::OsString>,
        old_module_root: Option<std::ffi::OsString>,
        old_instance_id: Option<std::ffi::OsString>,
        bundle_root: std::path::PathBuf,
    }

    impl Vst3EnvGuard {
        fn instrument() -> Self {
            let old_plugin_type_id = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_VST3_PLUGIN_TYPE_ID");
            let old_module_root = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_VST3_MODULE_ROOT");
            let old_instance_id = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_VST3_INSTANCE_ID");
            let bundle_root = temp_vst3_bundle_root("instrument");
            write_test_vst3_bundle(&bundle_root);
            unsafe {
                std::env::set_var(
                    "SIGNAL_PLUGIN_SANDBOX_VST3_PLUGIN_TYPE_ID",
                    "plugin:vst3:instrument",
                );
                std::env::set_var(
                    "SIGNAL_PLUGIN_SANDBOX_VST3_MODULE_ROOT",
                    bundle_root.as_os_str(),
                );
                std::env::set_var(
                    "SIGNAL_PLUGIN_SANDBOX_VST3_INSTANCE_ID",
                    "instance:sandbox:vst3:test",
                );
            }
            Self {
                old_plugin_type_id,
                old_module_root,
                old_instance_id,
                bundle_root,
            }
        }
    }

    impl Drop for Vst3EnvGuard {
        fn drop(&mut self) {
            restore_env(
                "SIGNAL_PLUGIN_SANDBOX_VST3_PLUGIN_TYPE_ID",
                self.old_plugin_type_id.as_ref(),
            );
            restore_env(
                "SIGNAL_PLUGIN_SANDBOX_VST3_MODULE_ROOT",
                self.old_module_root.as_ref(),
            );
            restore_env(
                "SIGNAL_PLUGIN_SANDBOX_VST3_INSTANCE_ID",
                self.old_instance_id.as_ref(),
            );
            let _ = fs::remove_dir_all(&self.bundle_root);
        }
    }

    struct AuEnvGuard {
        old_plugin_type_id: Option<std::ffi::OsString>,
        old_bundle_root: Option<std::ffi::OsString>,
        old_instance_id: Option<std::ffi::OsString>,
        bundle_root: std::path::PathBuf,
    }

    struct Lv2EnvGuard {
        old_plugin_type_id: Option<std::ffi::OsString>,
        old_bundle_root: Option<std::ffi::OsString>,
        old_instance_id: Option<std::ffi::OsString>,
        bundle_root: std::path::PathBuf,
    }

    impl Lv2EnvGuard {
        fn instrument() -> Self {
            let old_plugin_type_id = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_LV2_PLUGIN_TYPE_ID");
            let old_bundle_root = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_LV2_BUNDLE_ROOT");
            let old_instance_id = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_LV2_INSTANCE_ID");
            let bundle_root = temp_lv2_bundle_root("instrument");
            write_test_lv2_bundle(&bundle_root);
            unsafe {
                std::env::set_var(
                    "SIGNAL_PLUGIN_SANDBOX_LV2_PLUGIN_TYPE_ID",
                    "plugin:lv2:linux-synth",
                );
                std::env::set_var(
                    "SIGNAL_PLUGIN_SANDBOX_LV2_BUNDLE_ROOT",
                    bundle_root.as_os_str(),
                );
                std::env::set_var(
                    "SIGNAL_PLUGIN_SANDBOX_LV2_INSTANCE_ID",
                    "instance:sandbox:lv2:test",
                );
            }
            Self {
                old_plugin_type_id,
                old_bundle_root,
                old_instance_id,
                bundle_root,
            }
        }
    }

    impl Drop for Lv2EnvGuard {
        fn drop(&mut self) {
            restore_env(
                "SIGNAL_PLUGIN_SANDBOX_LV2_PLUGIN_TYPE_ID",
                self.old_plugin_type_id.as_ref(),
            );
            restore_env(
                "SIGNAL_PLUGIN_SANDBOX_LV2_BUNDLE_ROOT",
                self.old_bundle_root.as_ref(),
            );
            restore_env(
                "SIGNAL_PLUGIN_SANDBOX_LV2_INSTANCE_ID",
                self.old_instance_id.as_ref(),
            );
            let _ = fs::remove_dir_all(&self.bundle_root);
        }
    }

    impl AuEnvGuard {
        fn instrument() -> Self {
            let old_plugin_type_id = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_AU_PLUGIN_TYPE_ID");
            let old_bundle_root = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_AU_BUNDLE_ROOT");
            let old_instance_id = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_AU_INSTANCE_ID");
            let bundle_root = temp_au_bundle_root("instrument");
            write_test_au_bundle(&bundle_root);
            unsafe {
                std::env::set_var(
                    "SIGNAL_PLUGIN_SANDBOX_AU_PLUGIN_TYPE_ID",
                    "plugin:au:instrument",
                );
                std::env::set_var(
                    "SIGNAL_PLUGIN_SANDBOX_AU_BUNDLE_ROOT",
                    bundle_root.as_os_str(),
                );
                std::env::set_var(
                    "SIGNAL_PLUGIN_SANDBOX_AU_INSTANCE_ID",
                    "instance:sandbox:au:test",
                );
            }
            Self {
                old_plugin_type_id,
                old_bundle_root,
                old_instance_id,
                bundle_root,
            }
        }
    }

    impl Drop for AuEnvGuard {
        fn drop(&mut self) {
            restore_env(
                "SIGNAL_PLUGIN_SANDBOX_AU_PLUGIN_TYPE_ID",
                self.old_plugin_type_id.as_ref(),
            );
            restore_env(
                "SIGNAL_PLUGIN_SANDBOX_AU_BUNDLE_ROOT",
                self.old_bundle_root.as_ref(),
            );
            restore_env(
                "SIGNAL_PLUGIN_SANDBOX_AU_INSTANCE_ID",
                self.old_instance_id.as_ref(),
            );
            let _ = fs::remove_dir_all(&self.bundle_root);
        }
    }

    fn temp_vst3_bundle_root(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir()
            .join(format!("signal-plugin-sandbox-{label}-{unique}"))
            .join("Signal Instrument.vst3");
        fs::create_dir_all(root.join("Contents").join("Resources"))
            .expect("temp vst3 bundle should be created");
        root
    }

    fn temp_au_bundle_root(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir()
            .join(format!("signal-plugin-sandbox-au-{label}-{unique}"))
            .join("Signal Instrument.component");
        fs::create_dir_all(root.join("Contents").join("Resources"))
            .expect("temp au bundle should be created");
        root
    }

    fn temp_lv2_bundle_root(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir()
            .join(format!("signal-plugin-sandbox-lv2-{label}-{unique}"))
            .join("Signal Linux Synth.lv2");
        fs::create_dir_all(&root).expect("temp lv2 bundle should be created");
        root
    }

    fn write_test_vst3_bundle(bundle_root: &std::path::Path) {
        fs::create_dir_all(bundle_root.join("Contents").join("Resources"))
            .expect("vst3 metadata resources should be created");
        fs::write(
            bundle_root.join("Contents").join("Info.plist"),
            vst3_info_plist_contents(
                concat!(
                    "plugin_type_id=plugin:vst3:instrument\n",
                    "class_id=7E1D8F8A4D874D56A2C44DE250100001\n",
                    "controller_class_id=7E1D8F8A4D874D56A2C44DE250100002\n",
                    "category=Instrument\n",
                    "vendor=Signal\n",
                    "name=Signal Instrument VST3 Plugin\n",
                    "version=0.1.0\n",
                    "audio_inputs=0\n",
                    "audio_outputs=2\n",
                    "midi_inputs=1\n",
                    "midi_outputs=0\n",
                    "features=Instrument,Analyzer\n"
                ),
                bundle_root,
            ),
        )
        .expect("vst3 info plist should be written");
        fs::write(
            bundle_root
                .join("Contents")
                .join("Resources")
                .join("moduleinfo.json"),
            vst3_moduleinfo_contents(concat!(
                "plugin_type_id=plugin:vst3:instrument\n",
                "class_id=7E1D8F8A4D874D56A2C44DE250100001\n",
                "controller_class_id=7E1D8F8A4D874D56A2C44DE250100002\n",
                "category=Instrument\n",
                "vendor=Signal\n",
                "name=Signal Instrument VST3 Plugin\n",
                "version=0.1.0\n",
                "audio_inputs=0\n",
                "audio_outputs=2\n",
                "midi_inputs=1\n",
                "midi_outputs=0\n",
                "features=Instrument,Analyzer\n"
            )),
        )
        .expect("vst3 moduleinfo should be written");
    }

    fn vst3_info_plist_contents(metadata: &str, bundle_root: &std::path::Path) -> String {
        let mut plugin_type_id = "";
        let mut name = "Signal VST3 Plugin";
        let mut version = "0.1.0";
        let mut audio_inputs = "2";
        let mut audio_outputs = "2";
        let mut midi_inputs = "0";
        let mut midi_outputs = "0";
        let mut features = "";

        for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "plugin_type_id" => plugin_type_id = value.trim(),
                "name" => name = value.trim(),
                "version" => version = value.trim(),
                "audio_inputs" => audio_inputs = value.trim(),
                "audio_outputs" => audio_outputs = value.trim(),
                "midi_inputs" => midi_inputs = value.trim(),
                "midi_outputs" => midi_outputs = value.trim(),
                "features" => features = value.trim(),
                _ => {}
            }
        }

        let executable_name = bundle_root
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(name);
        let feature_array = features
            .split(',')
            .map(str::trim)
            .filter(|feature| !feature.is_empty())
            .map(|feature| format!("    <string>{feature}</string>"))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>CFBundleExecutable</key>\n\
  <string>{executable_name}</string>\n\
  <key>CFBundleIdentifier</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>CFBundleName</key>\n\
  <string>{name}</string>\n\
  <key>CFBundlePackageType</key>\n\
  <string>BNDL</string>\n\
  <key>CFBundleShortVersionString</key>\n\
  <string>{version}</string>\n\
  <key>SignalPluginTypeId</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>SignalAudioInputs</key>\n\
  <integer>{audio_inputs}</integer>\n\
  <key>SignalAudioOutputs</key>\n\
  <integer>{audio_outputs}</integer>\n\
  <key>SignalMidiInputs</key>\n\
  <integer>{midi_inputs}</integer>\n\
  <key>SignalMidiOutputs</key>\n\
  <integer>{midi_outputs}</integer>\n\
  <key>SignalFeatures</key>\n\
  <array>\n\
{feature_array}\n\
  </array>\n\
</dict>\n\
</plist>\n"
        )
    }

    fn vst3_moduleinfo_contents(metadata: &str) -> String {
        let mut class_id = "";
        let mut controller_class_id = "";
        let mut category = "Fx";
        let mut vendor = "Signal";
        let mut name = "Signal VST3 Plugin";
        let mut version = "0.1.0";

        for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "class_id" => class_id = value.trim(),
                "controller_class_id" => controller_class_id = value.trim(),
                "category" => category = value.trim(),
                "vendor" => vendor = value.trim(),
                "name" => name = value.trim(),
                "version" => version = value.trim(),
                _ => {}
            }
        }

        let subcategory = if category.eq_ignore_ascii_case("Instrument") {
            "Instrument"
        } else {
            "Fx"
        };
        let controller_class = if controller_class_id.is_empty()
            || controller_class_id.eq_ignore_ascii_case("none")
        {
            String::new()
        } else {
            format!(
                ",\n    {{\n      \"CID\": \"{controller_class_id}\",\n      \"Category\": \"Component Controller Class\",\n      \"Name\": \"{name}\",\n      \"Vendor\": \"{vendor}\",\n      \"Version\": \"{version}\",\n      \"Sub Categories\": [\"{subcategory}\"]\n    }}"
            )
        };

        format!(
            "{{\n  \"Name\": \"{name}\",\n  \"Version\": \"{version}\",\n  \"Factory Info\": {{\n    \"Vendor\": \"{vendor}\",\n    \"URL\": \"https://signal.dev\",\n    \"E-Mail\": \"\"\n  }},\n  \"Classes\": [\n    {{\n      \"CID\": \"{class_id}\",\n      \"Category\": \"Audio Module Class\",\n      \"Name\": \"{name}\",\n      \"Vendor\": \"{vendor}\",\n      \"Version\": \"{version}\",\n      \"Sub Categories\": [\"{subcategory}\"]\n    }}{controller_class}\n  ]\n}}\n"
        )
    }

    fn write_test_au_bundle(bundle_root: &std::path::Path) {
        fs::create_dir_all(bundle_root.join("Contents")).expect("au bundle contents should exist");
        fs::write(
            bundle_root.join("Contents").join("Info.plist"),
            au_info_plist_contents(concat!(
                "plugin_type_id=plugin:au:instrument\n",
                "component_type=aumu\n",
                "component_subtype=sigi\n",
                "manufacturer_code=sigl\n",
                "vendor=Signal\n",
                "name=Signal Instrument AU Plugin\n",
                "version=0.1.0\n",
                "audio_inputs=0\n",
                "audio_outputs=2\n",
                "midi_inputs=1\n",
                "midi_outputs=0\n",
                "features=Instrument,Analyzer\n"
            )),
        )
        .expect("au info plist should be written");
    }

    fn au_info_plist_contents(metadata: &str) -> String {
        let mut plugin_type_id = "";
        let mut component_type = "";
        let mut component_subtype = "";
        let mut manufacturer_code = "";
        let mut vendor = "Signal";
        let mut name = "Signal AU Plugin";
        let mut version = "0.1.0";
        let mut audio_inputs = "2";
        let mut audio_outputs = "2";
        let mut midi_inputs = "0";
        let mut midi_outputs = "0";
        let mut features = "";

        for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "plugin_type_id" => plugin_type_id = value.trim(),
                "component_type" => component_type = value.trim(),
                "component_subtype" => component_subtype = value.trim(),
                "manufacturer_code" => manufacturer_code = value.trim(),
                "vendor" => vendor = value.trim(),
                "name" => name = value.trim(),
                "version" => version = value.trim(),
                "audio_inputs" => audio_inputs = value.trim(),
                "audio_outputs" => audio_outputs = value.trim(),
                "midi_inputs" => midi_inputs = value.trim(),
                "midi_outputs" => midi_outputs = value.trim(),
                "features" => features = value.trim(),
                _ => {}
            }
        }

        let feature_array = features
            .split(',')
            .map(str::trim)
            .filter(|feature| !feature.is_empty())
            .map(|feature| format!("    <string>{feature}</string>"))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>AudioComponents</key>\n\
  <array>\n\
    <dict>\n\
      <key>manufacturer</key>\n\
      <string>{manufacturer_code}</string>\n\
      <key>name</key>\n\
      <string>{vendor}: {name}</string>\n\
      <key>sandboxSafe</key>\n\
      <false/>\n\
      <key>subtype</key>\n\
      <string>{component_subtype}</string>\n\
      <key>type</key>\n\
      <string>{component_type}</string>\n\
      <key>version</key>\n\
      <integer>1</integer>\n\
    </dict>\n\
  </array>\n\
  <key>CFBundleExecutable</key>\n\
  <string>{name}</string>\n\
  <key>CFBundleIdentifier</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>CFBundleName</key>\n\
  <string>{name}</string>\n\
  <key>CFBundlePackageType</key>\n\
  <string>BNDL</string>\n\
  <key>CFBundleShortVersionString</key>\n\
  <string>{version}</string>\n\
  <key>SignalPluginTypeId</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>SignalVendor</key>\n\
  <string>{vendor}</string>\n\
  <key>SignalDisplayName</key>\n\
  <string>{name}</string>\n\
  <key>SignalAudioInputs</key>\n\
  <integer>{audio_inputs}</integer>\n\
  <key>SignalAudioOutputs</key>\n\
  <integer>{audio_outputs}</integer>\n\
  <key>SignalMidiInputs</key>\n\
  <integer>{midi_inputs}</integer>\n\
  <key>SignalMidiOutputs</key>\n\
  <integer>{midi_outputs}</integer>\n\
  <key>SignalFeatures</key>\n\
  <array>\n\
{feature_array}\n\
  </array>\n\
</dict>\n\
</plist>\n"
        )
    }

    fn write_test_lv2_bundle(bundle_root: &std::path::Path) {
        fs::write(
            bundle_root.join("manifest.ttl"),
            concat!(
                "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n",
                "@prefix signal: <https://signal.dev/ns/lv2#> .\n",
                "signal:plugin_type_id \"plugin:lv2:linux-synth\" .\n",
                "signal:plugin_uri \"https://signal.dev/plugins/lv2/linux-synth\" .\n",
                "signal:vendor \"Signal\" .\n",
                "signal:name \"Signal Linux Synth LV2 Plugin\" .\n",
                "signal:version \"0.1.0\" .\n",
                "signal:audio_inputs \"0\" .\n",
                "signal:audio_outputs \"2\" .\n",
                "signal:midi_inputs \"1\" .\n",
                "signal:midi_outputs \"0\" .\n",
                "signal:required_feature \"http://lv2plug.in/ns/ext/urid#map\" .\n",
                "signal:required_feature \"http://lv2plug.in/ns/ext/worker#schedule\" .\n",
                "signal:supported_extension \"http://lv2plug.in/ns/ext/patch#Message\" .\n",
                "signal:supported_extension \"http://lv2plug.in/ns/ext/state#state\" .\n",
                "signal:feature \"Instrument\" .\n",
                "signal:feature \"Analyzer\" .\n"
            ),
        )
        .expect("lv2 manifest should be written");
    }

    fn restore_env(key: &str, value: Option<&std::ffi::OsString>) {
        unsafe {
            if let Some(value) = value {
                std::env::set_var(key, value);
            } else {
                std::env::remove_var(key);
            }
        }
    }

    #[test]
    fn broker_serves_startup_status_demo_and_shutdown_receipts() {
        let mut broker = SandboxBrokerProcess::default();
        let mut output = Vec::new();

        broker
            .serve(Cursor::new("status\nrun-demo\nshutdown\n"), &mut output)
            .expect("broker serve should succeed");

        let rendered = String::from_utf8(output).expect("broker output should be utf8");
        assert!(rendered.contains("state=starting"));
        assert!(rendered.contains("state=ready"));
        assert!(rendered.contains("state=attached"), "{rendered}");
        assert!(rendered.contains("detail=lease_attached"));
        assert!(rendered.contains("state=running"));
        assert!(rendered.contains("state=teardown_complete"));
        assert!(rendered.contains("detail=lease_cleanup_ok"));
        assert!(rendered.contains("state=shutdown"));
    }

    #[test]
    fn broker_emits_timed_out_receipt_for_timeout_demo() {
        let mut broker = SandboxBrokerProcess::default();
        let mut output = Vec::new();

        broker
            .serve(Cursor::new("run-timeout-demo\nshutdown\n"), &mut output)
            .expect("timeout broker serve should succeed");

        let rendered = String::from_utf8(output).expect("broker output should be utf8");
        assert!(rendered.contains("state=timed_out"));
        assert!(rendered.contains("detail=lease_attached_block_processing_timeout"));
        assert!(rendered.contains("detail=lease_cleanup_ok_after_timeout"));
    }

    #[test]
    fn broker_emits_vst3_flavored_receipts() {
        let _guard = Vst3EnvGuard::instrument();
        let mut broker = SandboxBrokerProcess::default();
        let mut output = Vec::new();

        broker
            .serve(Cursor::new("run-vst3\nshutdown\n"), &mut output)
            .expect("vst3 broker serve should succeed");

        let rendered = String::from_utf8(output).expect("broker output should be utf8");
        assert!(rendered.contains("state=attached"));
        assert!(rendered.contains(
            "detail=lease_attached|vst3:instance=instance:sandbox:vst3:test_state_stored="
        ));
        assert!(rendered.contains("component=Signal_Instrument_VST3_Plugin"));
        assert!(rendered.contains("state=running"));
        assert!(rendered.contains("stream_index=1"));
        assert!(rendered.contains("stream_index=3"));
        assert!(rendered.contains("block_sequence=2"));
        assert!(rendered.contains("audio_outputs=2"));
        assert!(rendered.contains("parameter_events=4"));
        assert!(rendered.contains("midi_events=2"));
        assert!(rendered.contains("parameter_signature="));
        assert!(rendered.contains("parameter_application_order=block2:parameters(1)->midi(2)"));
        assert!(rendered.contains(
            "event_packet_order=block2[param_packets=1,midi_packets=2,apply=parameters_then_midi]"
        ));
        assert!(rendered.contains("automation_delta=block0:delta[param=2,midi=1,baseline="));
        assert!(rendered.contains("next_state_digest="));
        assert!(rendered.contains("state_transition=applied"));
        assert!(rendered.contains("state=teardown_complete"));
        assert!(rendered.contains("flushed_state_bytes="));
    }

    #[test]
    fn broker_emits_au_flavored_receipts() {
        let _guard = AuEnvGuard::instrument();
        let mut broker = SandboxBrokerProcess::default();
        let mut output = Vec::new();

        broker
            .serve(Cursor::new("run-au\nshutdown\n"), &mut output)
            .expect("au broker serve should succeed");

        let rendered = String::from_utf8(output).expect("broker output should be utf8");
        assert!(rendered.contains("state=attached"));
        assert!(rendered
            .contains("detail=lease_attached|au:instance=instance:sandbox:au:test_state_stored=1"));
        assert!(rendered.contains("component=Signal_Instrument_AU_Plugin"));
        assert!(rendered.contains("component_type=aumu"));
        assert!(rendered.contains("component_subtype=sigi"));
        assert!(rendered.contains("manufacturer=sigl"));
        assert!(rendered.contains("state=running"));
        assert!(rendered.contains("processed_blocks=0|au:lifecycle_ready|state_snapshot=stored"));
        assert!(rendered.contains("state=teardown_complete"));
        assert!(rendered.contains("lease_cleanup_ok|au:flushed_state_bytes="));
    }

    #[test]
    fn broker_emits_lv2_flavored_receipts() {
        let _guard = Lv2EnvGuard::instrument();
        let mut broker = SandboxBrokerProcess::default();
        let mut output = Vec::new();

        broker
            .serve(
                Cursor::new("attach-lv2\nstream-lv2\nteardown-lv2\nshutdown\n"),
                &mut output,
            )
            .expect("lv2 broker serve should succeed");

        let rendered = String::from_utf8(output).expect("broker output should be utf8");
        assert!(rendered.contains("state=attached"));
        assert!(rendered.contains("detail=lease_attached|lv2:plugin_type=plugin:lv2:linux-synth"));
        assert!(rendered.contains("uri=https://signal.dev/plugins/lv2/linux-synth"));
        assert!(rendered.contains("worker=WorkerRequiredAvailable"));
        assert!(rendered.contains("urid=Negotiated"));
        assert!(rendered.contains("patch=Supported"));
        assert!(rendered.contains("negotiation=Negotiated"));
        assert!(rendered.contains("state=running"));
        assert!(rendered.contains("stream_index=1"));
        assert!(rendered.contains("stream_index=3"));
        assert!(rendered.contains("execution_complete"));
        assert!(rendered.contains("processed_blocks=3"));
        assert!(rendered.contains("last_block_sequence=2"));
        assert!(rendered.contains("block_frames=256"));
        assert!(rendered.contains("stream_order=block0[frames=128,patch=1,midi=0,completion=Applied];block1[frames=192,patch=2,midi=1,completion=Applied];block2[frames=256,patch=1,midi=2,completion=Applied]"));
        assert!(rendered.contains("patch_messages=2"));
        assert!(rendered.contains("midi_events=1"));
        assert!(rendered.contains("completion=Applied"));
        assert!(rendered.contains("state=teardown_complete"));
        assert!(rendered.contains("lease_cleanup_ok|lv2:instance=instance:sandbox:lv2:test"));
        assert!(rendered.contains("teardown=prepared_negotiation_flushed"));
    }

    #[test]
    fn broker_streams_vst3_execution_without_tearing_down_attached_session() {
        let _guard = Vst3EnvGuard::instrument();
        let mut broker = SandboxBrokerProcess::default();
        let mut output = Vec::new();

        broker
            .serve(
                Cursor::new("attach-vst3\nstream-vst3\nstream-vst3\nteardown-vst3\nshutdown\n"),
                &mut output,
            )
            .expect("vst3 broker stream should succeed");

        let rendered = String::from_utf8(output).expect("broker output should be utf8");
        assert!(rendered.contains("state=attached"));
        assert!(rendered.contains("stream_index=1"));
        assert!(rendered.contains("stream_index=2"));
        assert!(rendered.contains("stream_index=3"));
        assert!(rendered.contains("parameter_events=4"));
        assert!(rendered.contains("midi_events=2"));
        assert!(rendered.contains("parameter_signature="));
        assert!(rendered.contains("application_order=block0:parameters(2)->midi(1);block1:parameters(4)->midi(0);block2:parameters(1)->midi(2)"));
        assert!(rendered.contains("packet_order=block0[param_packets=2,midi_packets=1,apply=parameters_then_midi];block1[param_packets=4,midi_packets=0,apply=parameters_then_midi];block2[param_packets=1,midi_packets=2,apply=parameters_then_midi]"));
        assert!(rendered.contains("automation_delta=block0:delta[param=3,midi=2,baseline="));
        assert!(rendered.contains("next_state_digest="));
        assert!(rendered.contains("state_transition=applied"));
        assert!(rendered.contains("execution_complete"));
        assert!(rendered.contains("execution_runs=2"));
        assert!(rendered.contains("continuity=carried_forward"));
        assert!(rendered.contains("continued_from="));
        assert!(rendered.contains("state=teardown_complete"));
    }

    #[test]
    fn broker_resets_vst3_continuity_after_teardown_and_reattach() {
        let _guard = Vst3EnvGuard::instrument();
        let mut broker = SandboxBrokerProcess::default();
        let mut output = Vec::new();

        broker
            .serve(
                Cursor::new(
                    "attach-vst3\nstream-vst3\nstream-vst3\nteardown-vst3\nattach-vst3\nstream-vst3\nstream-vst3\nteardown-vst3\nshutdown\n",
                ),
                &mut output,
            )
            .expect("vst3 broker reattach stream should succeed");

        let rendered = String::from_utf8(output).expect("broker output should be utf8");
        assert!(rendered.contains("execution_runs=1"));
        assert!(rendered.contains("continuity=fresh"));
        assert!(rendered.contains("continued_from=none"));
        assert!(rendered.contains("execution_runs=2"));
        assert!(rendered.contains("continuity=carried_forward"));
        assert!(rendered.contains("automation_delta=block0:delta[param=2,midi=1,baseline="));
        assert!(rendered.contains("automation_delta=block0:delta[param=3,midi=2,baseline="));
        assert!(rendered.contains("state=teardown_complete"));
    }

    #[test]
    fn broker_refreshes_vst3_state_without_teardown() {
        let _guard = Vst3EnvGuard::instrument();
        let mut broker = SandboxBrokerProcess::default();
        let mut output = Vec::new();

        broker
            .serve(
                Cursor::new(
                    "attach-vst3\nstream-vst3\nstream-vst3\nrefresh-vst3\nstream-vst3\nteardown-vst3\nshutdown\n",
                ),
                &mut output,
            )
            .expect("vst3 broker refresh stream should succeed");

        let rendered = String::from_utf8(output).expect("broker output should be utf8");
        assert!(rendered.contains("execution_runs=2"));
        assert!(rendered.contains("continuity=carried_forward"));
        assert!(rendered.contains("refresh_cycle=state_store"));
        assert!(rendered.contains("continuity_reset=refreshed"));
        assert!(rendered.contains("execution_runs=1"));
        assert!(rendered.contains("continuity=fresh"));
        assert!(rendered.contains("continued_from=none"));
        assert!(rendered.contains("automation_delta=block0:delta[param=2,midi=1,baseline="));
        assert!(rendered.contains("automation_delta=block0:delta[param=3,midi=2,baseline="));
        assert!(rendered.contains("state=teardown_complete"));
    }

    #[test]
    fn broker_reports_recoverable_vst3_timeout_after_refresh_cycle() {
        let _guard = Vst3EnvGuard::instrument();
        let mut broker = SandboxBrokerProcess::default();
        let mut output = Vec::new();

        broker
            .serve(
                Cursor::new(
                    "attach-vst3\nstream-vst3\nstream-vst3\nrefresh-vst3\nstream-vst3\ntimeout-vst3\nteardown-vst3\nshutdown\n",
                ),
                &mut output,
            )
            .expect("vst3 broker timeout stream should succeed");

        let rendered = String::from_utf8(output).expect("broker output should be utf8");
        assert!(rendered.contains("refresh_cycle=state_store"));
        assert!(rendered.contains("continuity_reset=refreshed"));
        assert!(rendered.contains("execution_interrupted"));
        assert!(rendered.contains("timeout=recoverable"));
        assert!(rendered.contains("resume_hint=refresh_or_stream"));
        assert!(rendered.contains("execution_runs=1"));
        assert!(rendered.contains("state=teardown_complete"));
    }

    #[test]
    fn broker_supports_attach_then_teardown_demo_receipts() {
        let mut broker = SandboxBrokerProcess::default();
        let mut output = Vec::new();

        broker
            .serve(
                Cursor::new("attach-demo\nstatus\nteardown-demo\nshutdown\n"),
                &mut output,
            )
            .expect("attach broker serve should succeed");

        let rendered = String::from_utf8(output).expect("broker output should be utf8");
        assert!(rendered.contains("state=attached"));
        assert!(rendered.contains("detail=lease_attached"));
        assert!(rendered.contains("state=teardown_complete"));
        assert!(rendered.contains("detail=lease_cleanup_ok"));
        assert!(rendered.contains("state=shutdown"));
    }
}
