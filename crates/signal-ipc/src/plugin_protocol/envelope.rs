use crate::{CorrelationId, MessageKind, RuntimeDomain, RuntimeMessage};

use super::{PluginMessageName, PluginMessagePayload};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginMessageEnvelope {
    pub message: RuntimeMessage,
    pub payload: PluginMessagePayload,
}

impl PluginMessageEnvelope {
    pub fn command(
        name: PluginMessageName,
        correlation_id: impl Into<String>,
        payload: PluginMessagePayload,
    ) -> Self {
        Self {
            message: RuntimeMessage {
                domain: RuntimeDomain::Plugin,
                kind: MessageKind::Command,
                name: name.as_str().into(),
                correlation_id: Some(CorrelationId::new(correlation_id)),
            },
            payload,
        }
    }

    pub fn response(
        name: PluginMessageName,
        correlation_id: CorrelationId,
        payload: PluginMessagePayload,
    ) -> Self {
        Self {
            message: RuntimeMessage::response(RuntimeDomain::Plugin, name.as_str(), correlation_id),
            payload,
        }
    }

    pub fn event(
        name: PluginMessageName,
        correlation_id: Option<CorrelationId>,
        payload: PluginMessagePayload,
    ) -> Self {
        Self {
            message: RuntimeMessage::event(RuntimeDomain::Plugin, name.as_str(), correlation_id),
            payload,
        }
    }
}
