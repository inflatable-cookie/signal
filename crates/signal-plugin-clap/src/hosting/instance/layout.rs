/// Main-bus stereo port layout summary for a hosted instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClapHostedPortLayout {
    /// Channel count of the main input bus (0 = none).
    pub main_input_channels: u16,
    /// Channel count of the main output bus (0 = none).
    pub main_output_channels: u16,
}

impl ClapHostedPortLayout {
    /// Phase 1 supports exactly a stereo main in + stereo main out effect.
    pub fn is_stereo_effect(&self) -> bool {
        self.main_input_channels == 2 && self.main_output_channels == 2
    }

    /// MIDI instrument layout supported by the current host: no main audio
    /// input and one stereo main output.
    pub fn is_stereo_instrument(&self) -> bool {
        self.main_input_channels == 0 && self.main_output_channels == 2
    }

    /// Whether the current stereo process session can host this layout.
    pub fn is_supported_stereo_processor(&self) -> bool {
        self.is_stereo_effect() || self.is_stereo_instrument()
    }

    /// Whether stereo inspection can safely drive this layout. The first
    /// input/output pair carries the inspection signal; extra input channels
    /// remain silent and extra outputs are ignored. Runtime hosting keeps the
    /// stricter exact-layout gate above.
    pub fn is_supported_stereo_inspection_processor(&self) -> bool {
        self.main_output_channels >= 2
            && (self.main_input_channels == 0 || self.main_input_channels >= 2)
    }
}

/// Lifecycle state of a hosted instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostedInstanceState {
    Created,
    Active,
}
