/// Logical subsystem a message belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDomain {
    /// Audio engine and graph processing.
    Engine,
    /// Plugin graph topology and routing.
    Graph,
    /// Hardware device and stream management.
    Hardware,
    /// Plugin sandbox lifecycle and processing.
    Plugin,
    /// Diagnostic reporting and health monitoring.
    Diagnostics,
}

/// Classification of a runtime message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    /// Requests that the receiver perform an action.
    Command,
    /// Carries unsolicited state or lifecycle information.
    Event,
    /// Answers a specific command, identified by [`CorrelationId`].
    Response,
}

/// Opaque identifier that correlates a command to its response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    /// Construct a correlation ID from any string-like value.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// A typed, domain-scoped runtime message.
///
/// Messages are the unit of communication across the runtime/plugin boundary.
/// Use the [`command`][Self::command], [`response`][Self::response], and
/// [`event`][Self::event] constructors rather than building the struct directly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMessage {
    /// Subsystem this message belongs to.
    pub domain: RuntimeDomain,
    /// Whether this is a command, event, or response.
    pub kind: MessageKind,
    /// Name of the specific message within the domain (e.g. `"sandbox.handshake"`).
    pub name: String,
    /// Correlation token, present on responses and optionally on events.
    pub correlation_id: Option<CorrelationId>,
}

impl RuntimeMessage {
    /// Construct a command message with no correlation ID set.
    pub fn command(domain: RuntimeDomain, name: impl Into<String>) -> Self {
        Self {
            domain,
            kind: MessageKind::Command,
            name: name.into(),
            correlation_id: None,
        }
    }

    /// Construct a response message correlated to an earlier command.
    pub fn response(
        domain: RuntimeDomain,
        name: impl Into<String>,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            domain,
            kind: MessageKind::Response,
            name: name.into(),
            correlation_id: Some(correlation_id),
        }
    }

    /// Construct an event message with an optional correlation ID.
    pub fn event(
        domain: RuntimeDomain,
        name: impl Into<String>,
        correlation_id: Option<CorrelationId>,
    ) -> Self {
        Self {
            domain,
            kind: MessageKind::Event,
            name: name.into(),
            correlation_id,
        }
    }
}
