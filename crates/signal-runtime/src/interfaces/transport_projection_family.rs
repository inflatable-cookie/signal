#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopRegion {
    pub start_samples: i64,
    pub end_samples: i64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransportProjection {
    pub playing: bool,
    pub timeline_position_samples: i64,
    pub tempo_bpm: f64,
    pub loop_state: Option<LoopRegion>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParameterEvent {
    pub target: String,
    pub sample_offset: usize,
    pub normalized_value: f32,
}

/// Runtime-owned batch of parameter changes accepted for one automation epoch.
///
/// Runtime stays authoritative for epoch assignment and block-boundary
/// application; callers supply only the logical event payload.
#[derive(Clone, Debug, PartialEq)]
pub struct ParameterBatch {
    pub epoch: u64,
    pub events: Vec<ParameterEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeAutomationInterpolation {
    Hold,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeAutomationResolution {
    pub ramp_step_samples: usize,
    pub max_sub_blocks: usize,
}

impl Default for RuntimeAutomationResolution {
    fn default() -> Self {
        Self {
            ramp_step_samples: 32,
            max_sub_blocks: 8,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAutomationTargetProjection {
    pub node_id: String,
    pub parameter_id: String,
}

impl RuntimeAutomationTargetProjection {
    pub fn parameter_path(&self) -> String {
        format!("{}.{}", self.node_id, self.parameter_id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeAutomationPointProjection {
    pub time_samples: i64,
    pub normalized_value: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeAutomationLaneProjection {
    pub automation_lane_id: String,
    pub target: RuntimeAutomationTargetProjection,
    pub base_normalized_value: f32,
    pub interpolation: RuntimeAutomationInterpolation,
    pub resolution: RuntimeAutomationResolution,
    pub point_count: usize,
    pub points: Vec<RuntimeAutomationPointProjection>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeAutomationProjection {
    pub lane_count: usize,
    pub point_count: usize,
    pub lanes: Vec<RuntimeAutomationLaneProjection>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionReceipt {
    pub accepted_epoch: u64,
    pub applied_at_block_boundary: bool,
}
