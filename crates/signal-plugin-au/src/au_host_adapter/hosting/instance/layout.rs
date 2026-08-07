/// Default main-element port layout reported by a hosted AU instance.
///
/// This is descriptive, not a declaration of every layout the unit supports:
/// Audio Units may report a mono default while accepting stereo during stream
/// format negotiation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuHostedPortLayout {
    /// Channel count of input element 0 (0 = none, e.g. instruments).
    pub main_input_channels: u16,
    /// Channel count of output element 0 (0 = none).
    pub main_output_channels: u16,
}

impl AuHostedPortLayout {
    /// Whether the default main elements are stereo input + stereo output.
    pub fn is_stereo_effect(&self) -> bool {
        self.main_input_channels == 2 && self.main_output_channels == 2
    }
}

/// Lifecycle state of a hosted instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostedInstanceState {
    Created,
    Active,
}
