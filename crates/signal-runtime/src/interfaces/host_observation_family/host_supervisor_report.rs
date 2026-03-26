use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeHostSupervisorReport {
    pub observation: RuntimeHostObservationReport,
    pub events: Vec<RuntimeEvent>,
}

impl RuntimeHostSupervisorReport {
    pub fn new(supervisor: RuntimeSupervisorReport, host_io: RuntimeHostIoSummary) -> Self {
        Self {
            observation: RuntimeHostObservationReport::new(supervisor.observation, host_io),
            events: supervisor.events,
        }
    }

    pub fn render_compact(&self) -> String {
        format!(
            "{} event_stream={}",
            self.observation.render_compact(),
            self.events.len()
        )
    }

    pub fn render_multiline(&self) -> String {
        format!(
            "{}\nevent_stream={}",
            self.observation.render_multiline(),
            self.events.len()
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            "{{\"observation\":{},\"event_stream\":{}}}",
            self.observation.render_json(),
            self.events.len()
        )
    }

    pub fn profiling_receipt(&self) -> RuntimeProfilingReceipt {
        build_runtime_profiling_receipt(
            &self.observation.observation,
            Some(&self.observation.host_io),
        )
    }

    pub fn soak_receipt(&self) -> RuntimeSoakReceipt {
        build_runtime_soak_receipt(&self.observation.observation, self.events.len())
    }
}
