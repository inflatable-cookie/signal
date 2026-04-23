/// Start/end boundaries for a timeline loop region (in samples).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopRegion {
    /// Loop start position in samples.
    pub start_samples: i64,
    /// Loop end position in samples.
    pub end_samples: i64,
}

/// Full transport state applied to the runtime each control cycle.
///
/// Pass to `apply_transport_projection()`.  The runtime computes the
/// per-block sample ranges from `timeline_position_samples` + block size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransportProjection {
    /// Whether the transport is currently playing.
    pub playing: bool,
    /// Current timeline position in samples.
    pub timeline_position_samples: i64,
    /// Current tempo in beats per minute.
    pub tempo_bpm: f64,
    /// Active loop region, if looping is enabled.
    pub loop_state: Option<LoopRegion>,
}

/// A single normalised parameter change with a sample-level offset within a
/// block.
#[derive(Clone, Debug, PartialEq)]
pub struct ParameterEvent {
    /// Target parameter path (node_id.parameter_id).
    pub target: String,
    /// Sample offset within the current block at which to apply the change.
    pub sample_offset: usize,
    /// Normalised value in the range [0.0, 1.0].
    pub normalized_value: f32,
}

/// Runtime-owned batch of parameter changes accepted for one automation epoch.
///
/// Runtime stays authoritative for epoch assignment and block-boundary
/// application; callers supply only the logical event payload.
#[derive(Clone, Debug, PartialEq)]
pub struct ParameterBatch {
    /// Epoch identifier assigned by the runtime.
    pub epoch: u64,
    /// Parameter change events in this batch.
    pub events: Vec<ParameterEvent>,
}

/// Interpolation strategy between automation points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeAutomationInterpolation {
    /// Value snaps immediately to the point value.
    Hold,
    /// Value ramps linearly between consecutive points.
    Linear,
}

/// Controls sub-block resolution for automation ramp application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeAutomationResolution {
    /// Minimum number of samples between ramp steps.
    pub ramp_step_samples: usize,
    /// Maximum number of sub-block segments per block.
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

/// Identifies the graph node and parameter targeted by an automation lane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAutomationTargetProjection {
    /// Graph node identifier.
    pub node_id: String,
    /// Parameter identifier within the node.
    pub parameter_id: String,
}

impl RuntimeAutomationTargetProjection {
    /// Returns the fully qualified parameter path as `node_id.parameter_id`.
    pub fn parameter_path(&self) -> String {
        format!("{}.{}", self.node_id, self.parameter_id)
    }
}

/// A single automation control point in timeline-sample coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeAutomationPointProjection {
    /// Timeline position in samples.
    pub time_samples: i64,
    /// Normalised parameter value at this point.
    pub normalized_value: f32,
}

/// One automation lane with its target, interpolation settings, and points.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeAutomationLaneProjection {
    /// Stable lane identifier.
    pub automation_lane_id: String,
    /// Target node and parameter for this lane.
    pub target: RuntimeAutomationTargetProjection,
    /// Normalised base value used when no automation point is active.
    pub base_normalized_value: f32,
    /// Interpolation mode between consecutive points.
    pub interpolation: RuntimeAutomationInterpolation,
    /// Sub-block resolution settings for ramp application.
    pub resolution: RuntimeAutomationResolution,
    /// Number of automation points in this lane.
    pub point_count: usize,
    /// Ordered list of automation control points.
    pub points: Vec<RuntimeAutomationPointProjection>,
}

/// Full automation state for one control cycle.
///
/// Pass to `apply_automation_projection()`.  The runtime merges this with the
/// current transport position to schedule per-block parameter events.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeAutomationProjection {
    /// Number of automation lanes.
    pub lane_count: usize,
    /// Total number of automation points across all lanes.
    pub point_count: usize,
    /// Automation lane projections.
    pub lanes: Vec<RuntimeAutomationLaneProjection>,
}

/// Acknowledgement returned after a projection is accepted by the runtime.
///
/// `accepted_epoch` identifies the projection version; `applied_at_block_boundary`
/// is `true` when the change took effect at the very next block boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionReceipt {
    /// Epoch identifier assigned to the accepted projection.
    pub accepted_epoch: u64,
    /// Whether the projection was applied at the next block boundary.
    pub applied_at_block_boundary: bool,
}
