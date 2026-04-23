use crate::{CorrelationId, MessageKind, RuntimeDomain, RuntimeMessage};

use super::{PluginMessageName, PluginMessagePayload};

/// A plugin sandbox protocol message: a typed [`RuntimeMessage`] header paired
/// with its [`PluginMessagePayload`].
///
/// Use the [`command`][Self::command], [`response`][Self::response], and
/// [`event`][Self::event] constructors to build envelopes with the correct
/// [`MessageKind`] and [`RuntimeDomain::Plugin`] domain set automatically.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginMessageEnvelope {
    /// The runtime message header (domain, kind, name, correlation ID).
    pub message: RuntimeMessage,
    /// The typed payload for this message.
    pub payload: PluginMessagePayload,
}

impl PluginMessageEnvelope {
    /// Construct a command envelope. The correlation ID is generated from
    /// `correlation_id` and stored in the header.
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

    /// Construct a response envelope correlated to an earlier command.
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

    /// Construct an event envelope with an optional correlation ID.
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
