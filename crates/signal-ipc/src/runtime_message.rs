#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDomain {
    Engine,
    Graph,
    Hardware,
    Plugin,
    Diagnostics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    Command,
    Event,
    Response,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMessage {
    pub domain: RuntimeDomain,
    pub kind: MessageKind,
    pub name: String,
    pub correlation_id: Option<CorrelationId>,
}

impl RuntimeMessage {
    pub fn command(domain: RuntimeDomain, name: impl Into<String>) -> Self {
        Self {
            domain,
            kind: MessageKind::Command,
            name: name.into(),
            correlation_id: None,
        }
    }

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
