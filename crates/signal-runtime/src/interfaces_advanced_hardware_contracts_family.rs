use crate::RuntimeExternalMidiDiscoveryState;

/// Overall readiness of the advanced hardware (DAW controller) graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdvancedHardwareGraphState {
    /// Backend or discovery service is unavailable.
    Unavailable,
    /// Discovery succeeded but no devices are connected.
    Empty,
    /// All devices are ready with no guarded conditions.
    Ready,
    /// One or more devices are in a guarded state.
    Guarded,
}

/// Whether a device is permitted to run scripted or extended control actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeScriptingSafeDevicePolicyPosture {
    /// Device does not support scripted actions.
    Unsupported,
    /// Scripted actions are permitted only within a bounded context.
    ContextOnly,
    /// Scripted actions are explicitly denied for this device.
    Denied,
    /// Scripted action access is guarded pending policy resolution.
    Guarded,
    /// Scripted actions are portable across contexts.
    Portable,
}

/// Whether a device's feedback channel is portable, guarded, or absent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeGuardedFeedbackChannelPosture {
    /// No feedback channel is available for this device.
    Unavailable,
    /// Feedback channel is present but in a guarded state.
    Guarded,
    /// Feedback channel is available and portable.
    Portable,
}

/// Category of action a DAW controller device can perform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdvancedHardwareActionClass {
    /// Device can send visual display feedback.
    DisplayFeedback,
    /// Device has motorised fader or knob feedback.
    MotorFeedback,
    /// Device has haptic feedback capability.
    HapticFeedback,
    /// Device supports bank/scene navigation.
    BankNavigation,
    /// Device supports macro trigger actions.
    MacroTrigger,
    /// Device supports device state observation.
    DeviceStateObservation,
}

/// Visual display feedback capability of a controller device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeDisplayTransportPosture {
    /// Device has no display.
    NotPresent,
    /// Display is present but in a guarded state.
    GuardedDisplay,
    /// Display supports text-only output.
    TextOnlyDisplay,
    /// Display supports paged/contextual content.
    PageAwareDisplay,
    /// Display is present but currently unavailable.
    UnavailableDisplay,
}

/// Content class presented on a device display.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeDisplayContentClass {
    /// No content is being displayed.
    NoDisplayContent,
    /// Display shows general status text.
    StatusText,
    /// Display shows parameter value text.
    ParameterValueText,
    /// Display shows meter bridge text.
    MeterBridgeText,
    /// Display shows a paged status view.
    PagedStatusView,
    /// Display is showing guarded vendor-specific content.
    GuardedVendorDisplay,
}

/// Motorized fader or knob transport capability of a device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeMotorTransportPosture {
    /// Device has no motor transport.
    NoMotorTransport,
    /// Motor transport is present but guarded.
    GuardedMotorTransport,
    /// Motor transport supports absolute position feedback.
    PositionMotorTransport,
    /// Motor transport supports bank-aware position feedback.
    BankAwareMotorTransport,
    /// Motor transport is present but currently unavailable.
    UnavailableMotorTransport,
}

/// Haptic feedback capability of a device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeHapticTransportPosture {
    /// Device has no haptic transport.
    NoHapticTransport,
    /// Haptic transport is present but guarded.
    GuardedHapticTransport,
    /// Haptic transport supports cue-only feedback.
    CueOnlyHapticTransport,
    /// Haptic transport supports full state-aware feedback.
    StateAwareHapticTransport,
    /// Haptic transport is present but currently unavailable.
    UnavailableHapticTransport,
}

/// Who authorises the feedback channel configuration for a device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdvancedControlFeedbackAuthority {
    /// Runtime uses its default feedback configuration.
    RuntimeDefault,
    /// Runtime has explicitly declared the feedback configuration.
    RuntimeDeclared,
    /// Host has forwarded the feedback configuration.
    HostForwarded,
    /// Device is advisory — runtime defers to device preference.
    DeviceAdvisory,
}

/// Resolved behaviour of the feedback channel after authority negotiation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdvancedControlFeedbackOutcome {
    /// Declared feedback configuration is preserved as-is.
    PreserveDeclaredFeedback,
    /// Feedback is collapsed to the guarded fallback configuration.
    CollapseToGuardedFeedback,
    /// Feedback is in observe-only mode with no active output.
    ObserveOnlyFeedback,
    /// Feedback transport is bypassed entirely.
    BypassFeedbackTransport,
    /// Feedback channel has entered a terminal failure state.
    TerminalFeedbackFailure,
}

/// Scene/bank mapping capability of a device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeSceneMappingPosture {
    /// Device does not support scene/bank mapping.
    NoSceneMapping,
    /// Scene mapping is present but guarded.
    GuardedSceneMapping,
    /// Scene mapping is contextual (depends on active session).
    ContextualSceneMapping,
    /// Scene mapping is portable across sessions.
    PortableSceneMapping,
    /// Scene mapping is present but currently unavailable.
    UnavailableSceneMapping,
}

/// Paged feedback capability of a device display.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeFeedbackPagePosture {
    /// Device does not support feedback pages.
    NoFeedbackPages,
    /// Feedback pages are present but guarded.
    GuardedFeedbackPages,
    /// Feedback pages support status-only content.
    StatusFeedbackPages,
    /// Feedback pages are scene-aware.
    SceneAwareFeedbackPages,
    /// Feedback pages are present but currently unavailable.
    UnavailableFeedbackPages,
}

/// Content class of a device feedback page.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeFeedbackPageClass {
    /// No feedback page class is assigned.
    NoFeedbackPageClass,
    /// Page shows general status information.
    StatusPage,
    /// Page shows parameter values.
    ParameterPage,
    /// Page shows meter data.
    MeterPage,
    /// Page shows scene/bank information.
    ScenePage,
    /// Page shows guarded vendor-specific content.
    GuardedVendorPage,
}

/// Whether a safe action graph is available for a device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeSafeActionGraphPosture {
    /// Device has no safe action graph.
    NoSafeActionGraph,
    /// Safe action graph is present but guarded.
    GuardedSafeActionGraph,
    /// Safe action graph supports transport actions.
    TransportSafeActionGraph,
    /// Safe action graph supports scene actions.
    SceneSafeActionGraph,
    /// Safe action graph is present but currently unavailable.
    UnavailableSafeActionGraph,
}

/// Who authorises the workflow action configuration for a control surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceWorkflowAuthority {
    /// Runtime uses its default workflow action configuration.
    RuntimeDefault,
    /// Runtime has explicitly declared the workflow action configuration.
    RuntimeDeclared,
    /// Host has forwarded the workflow action configuration.
    HostForwarded,
    /// Device is advisory — runtime defers to device preference.
    DeviceAdvisory,
}

/// Resolved behaviour of a device action after authority negotiation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeSafeActionOutcome {
    /// Declared action is preserved as-is.
    PreserveDeclaredAction,
    /// Action is collapsed to the guarded fallback.
    CollapseToGuardedAction,
    /// Action is in observe-only mode with no output.
    ObserveOnlyAction,
    /// Unsafe action is bypassed.
    BypassUnsafeAction,
    /// Action has entered a terminal failure state.
    TerminalActionFailure,
}

/// Capability flags for a DAW controller device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAdvancedHardwareCapabilitySummary {
    /// Whether the device supports display feedback output.
    pub supports_display_feedback: bool,
    /// Whether the device supports motor (fader/knob) feedback output.
    pub supports_motor_feedback: bool,
    /// Whether the device supports haptic feedback output.
    pub supports_haptic_feedback: bool,
    /// Whether the device supports bank/scene navigation.
    pub supports_bank_navigation: bool,
    /// Whether the device supports macro trigger actions.
    pub supports_macro_triggers: bool,
    /// Whether the device supports device state observation.
    pub supports_device_state_observation: bool,
    /// Enumerated action classes supported by the device.
    pub action_classes: Vec<RuntimeAdvancedHardwareActionClass>,
    /// Human-readable summary of the capability flags.
    pub summary: String,
}

/// Full descriptor for a single DAW controller device, including all posture,
/// authority, and capability fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAdvancedHardwareDeviceDescriptor {
    /// Stable identifier for the device.
    pub device_id: String,
    /// Human-readable name for the device.
    pub device_name: String,
    /// Scripting safety policy posture for the device.
    pub scripting_safe_posture: RuntimeScriptingSafeDevicePolicyPosture,
    /// Feedback channel portability posture.
    pub feedback_channel_posture: RuntimeGuardedFeedbackChannelPosture,
    /// Visual display transport posture.
    pub display_transport_posture: RuntimeDisplayTransportPosture,
    /// Content class currently shown on the device display.
    pub display_content_class: RuntimeDisplayContentClass,
    /// Motor (fader/knob) transport posture.
    pub motor_transport_posture: RuntimeMotorTransportPosture,
    /// Haptic feedback transport posture.
    pub haptic_transport_posture: RuntimeHapticTransportPosture,
    /// Authority that resolved the feedback channel configuration.
    pub feedback_authority: RuntimeAdvancedControlFeedbackAuthority,
    /// Resolved feedback channel outcome after authority negotiation.
    pub feedback_outcome: RuntimeAdvancedControlFeedbackOutcome,
    /// Scene/bank mapping posture.
    pub scene_mapping_posture: RuntimeSceneMappingPosture,
    /// Paged feedback posture.
    pub feedback_page_posture: RuntimeFeedbackPagePosture,
    /// Content class of the active feedback page.
    pub feedback_page_class: RuntimeFeedbackPageClass,
    /// Safe action graph posture.
    pub safe_action_graph_posture: RuntimeSafeActionGraphPosture,
    /// Authority that resolved the workflow action configuration.
    pub action_authority: RuntimeControlSurfaceWorkflowAuthority,
    /// Resolved safe action outcome after authority negotiation.
    pub safe_action_outcome: RuntimeSafeActionOutcome,
    /// Aggregated capability flags for this device.
    pub capability: RuntimeAdvancedHardwareCapabilitySummary,
    /// Human-readable summary of the device descriptor.
    pub summary: String,
}

/// Aggregate snapshot of all discovered advanced hardware (DAW controller)
/// devices and their capability counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAdvancedHardwareSnapshot {
    /// Current discovery phase of the external MIDI/hardware graph.
    pub discovery_state: RuntimeExternalMidiDiscoveryState,
    /// Overall readiness of the advanced hardware graph.
    pub graph_state: RuntimeAdvancedHardwareGraphState,
    /// Name of the backend provider supplying the device list.
    pub provider_name: String,
    /// Total number of discovered advanced hardware devices.
    pub device_count: usize,
    /// Number of devices with portable scripting posture.
    pub portable_device_count: usize,
    /// Number of devices with guarded scripting posture.
    pub guarded_device_count: usize,
    /// Number of devices with context-only scripting posture.
    pub context_only_device_count: usize,
    /// Number of devices with denied scripting posture.
    pub denied_device_count: usize,
    /// Number of devices that have an active feedback channel.
    pub feedback_channel_device_count: usize,
    /// Number of devices with a display transport.
    pub display_transport_device_count: usize,
    /// Number of devices with a motor transport.
    pub motor_transport_device_count: usize,
    /// Number of devices with haptic transport.
    pub haptic_transport_device_count: usize,
    /// Number of devices that support scene/bank mapping.
    pub scene_mapping_device_count: usize,
    /// Number of devices that support feedback pages.
    pub feedback_page_device_count: usize,
    /// Number of devices that have a safe action graph.
    pub safe_action_graph_device_count: usize,
    /// Per-device descriptors for all discovered advanced hardware devices.
    pub devices: Vec<RuntimeAdvancedHardwareDeviceDescriptor>,
    /// Human-readable summary of the snapshot.
    pub summary: String,
}
