use crate::RuntimeExternalMidiDiscoveryState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdvancedHardwareGraphState {
    Unavailable,
    Empty,
    Ready,
    Guarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeScriptingSafeDevicePolicyPosture {
    Unsupported,
    ContextOnly,
    Denied,
    Guarded,
    Portable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeGuardedFeedbackChannelPosture {
    Unavailable,
    Guarded,
    Portable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdvancedHardwareActionClass {
    DisplayFeedback,
    MotorFeedback,
    HapticFeedback,
    BankNavigation,
    MacroTrigger,
    DeviceStateObservation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeDisplayTransportPosture {
    NotPresent,
    GuardedDisplay,
    TextOnlyDisplay,
    PageAwareDisplay,
    UnavailableDisplay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeDisplayContentClass {
    NoDisplayContent,
    StatusText,
    ParameterValueText,
    MeterBridgeText,
    PagedStatusView,
    GuardedVendorDisplay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeMotorTransportPosture {
    NoMotorTransport,
    GuardedMotorTransport,
    PositionMotorTransport,
    BankAwareMotorTransport,
    UnavailableMotorTransport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeHapticTransportPosture {
    NoHapticTransport,
    GuardedHapticTransport,
    CueOnlyHapticTransport,
    StateAwareHapticTransport,
    UnavailableHapticTransport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdvancedControlFeedbackAuthority {
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    DeviceAdvisory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdvancedControlFeedbackOutcome {
    PreserveDeclaredFeedback,
    CollapseToGuardedFeedback,
    ObserveOnlyFeedback,
    BypassFeedbackTransport,
    TerminalFeedbackFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeSceneMappingPosture {
    NoSceneMapping,
    GuardedSceneMapping,
    ContextualSceneMapping,
    PortableSceneMapping,
    UnavailableSceneMapping,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeFeedbackPagePosture {
    NoFeedbackPages,
    GuardedFeedbackPages,
    StatusFeedbackPages,
    SceneAwareFeedbackPages,
    UnavailableFeedbackPages,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeFeedbackPageClass {
    NoFeedbackPageClass,
    StatusPage,
    ParameterPage,
    MeterPage,
    ScenePage,
    GuardedVendorPage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeSafeActionGraphPosture {
    NoSafeActionGraph,
    GuardedSafeActionGraph,
    TransportSafeActionGraph,
    SceneSafeActionGraph,
    UnavailableSafeActionGraph,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceWorkflowAuthority {
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    DeviceAdvisory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeSafeActionOutcome {
    PreserveDeclaredAction,
    CollapseToGuardedAction,
    ObserveOnlyAction,
    BypassUnsafeAction,
    TerminalActionFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAdvancedHardwareCapabilitySummary {
    pub supports_display_feedback: bool,
    pub supports_motor_feedback: bool,
    pub supports_haptic_feedback: bool,
    pub supports_bank_navigation: bool,
    pub supports_macro_triggers: bool,
    pub supports_device_state_observation: bool,
    pub action_classes: Vec<RuntimeAdvancedHardwareActionClass>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAdvancedHardwareDeviceDescriptor {
    pub device_id: String,
    pub device_name: String,
    pub scripting_safe_posture: RuntimeScriptingSafeDevicePolicyPosture,
    pub feedback_channel_posture: RuntimeGuardedFeedbackChannelPosture,
    pub display_transport_posture: RuntimeDisplayTransportPosture,
    pub display_content_class: RuntimeDisplayContentClass,
    pub motor_transport_posture: RuntimeMotorTransportPosture,
    pub haptic_transport_posture: RuntimeHapticTransportPosture,
    pub feedback_authority: RuntimeAdvancedControlFeedbackAuthority,
    pub feedback_outcome: RuntimeAdvancedControlFeedbackOutcome,
    pub scene_mapping_posture: RuntimeSceneMappingPosture,
    pub feedback_page_posture: RuntimeFeedbackPagePosture,
    pub feedback_page_class: RuntimeFeedbackPageClass,
    pub safe_action_graph_posture: RuntimeSafeActionGraphPosture,
    pub action_authority: RuntimeControlSurfaceWorkflowAuthority,
    pub safe_action_outcome: RuntimeSafeActionOutcome,
    pub capability: RuntimeAdvancedHardwareCapabilitySummary,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAdvancedHardwareSnapshot {
    pub discovery_state: RuntimeExternalMidiDiscoveryState,
    pub graph_state: RuntimeAdvancedHardwareGraphState,
    pub provider_name: String,
    pub device_count: usize,
    pub portable_device_count: usize,
    pub guarded_device_count: usize,
    pub context_only_device_count: usize,
    pub denied_device_count: usize,
    pub feedback_channel_device_count: usize,
    pub display_transport_device_count: usize,
    pub motor_transport_device_count: usize,
    pub haptic_transport_device_count: usize,
    pub scene_mapping_device_count: usize,
    pub feedback_page_device_count: usize,
    pub safe_action_graph_device_count: usize,
    pub devices: Vec<RuntimeAdvancedHardwareDeviceDescriptor>,
    pub summary: String,
}
