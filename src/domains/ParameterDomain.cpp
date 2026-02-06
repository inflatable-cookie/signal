#include "domains/ParameterDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/GraphEngine.hpp"
#include "core/GraphNodes.hpp"
#include "core/PluginInstance.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <algorithm>
#include <nlohmann/json.hpp>
#include <sstream>

namespace {
nlohmann::json makeScopePayload(
    const std::string& nodeId,
    const std::optional<std::string>& pluginInstanceId
) {
    nlohmann::json scopePayload = nlohmann::json::object();
    scopePayload["nodeId"] = nodeId;

    if (pluginInstanceId.has_value()) {
        scopePayload["pluginInstanceId"] = pluginInstanceId.value();
    }

    return scopePayload;
}
}

ParameterDomain::ParameterDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
}

void ParameterDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    if (env.domain != "parameter") {
        return;
    }

    if (env.kind != IpcKind::Command) {
        return;
    }

    if (!_engineHost) {
        emitError(
            env,
            session,
            "engine_unavailable",
            "EngineHost is unavailable",
            nlohmann::json::object()
        );
        return;
    }

    if (env.name == "requestDescriptors") {
        handleRequestDescriptors(env, session);
    } else if (env.name == "requestValues") {
        handleRequestValues(env, session);
    } else if (env.name == "setValue") {
        handleSetValue(env, session);
    } else {
        emitError(
            env,
            session,
            "unknown_command",
            "Unknown parameter command",
            nlohmann::json{{"name", env.name}}
        );
    }
}

std::optional<ParameterDomain::Scope> ParameterDomain::parseScope(const nlohmann::json& payload) const {
    if (!payload.contains("scope") || !payload["scope"].is_object()) {
        return std::nullopt;
    }

    const auto& scopeJson = payload["scope"];
    if (!scopeJson.contains("nodeId") || !scopeJson["nodeId"].is_string()) {
        return std::nullopt;
    }

    Scope scope;
    scope.nodeId = scopeJson["nodeId"].get<std::string>();

    if (scopeJson.contains("pluginInstanceId") && scopeJson["pluginInstanceId"].is_string()) {
        scope.pluginInstanceId = scopeJson["pluginInstanceId"].get<std::string>();
    }

    return scope;
}

PluginNode* ParameterDomain::resolvePluginNode(const Scope& scope) const {
    GraphNode* node = _engineHost->graphEngine().findNode(scope.nodeId);
    if (!node) {
        return nullptr;
    }

    return dynamic_cast<PluginNode*>(node);
}

std::optional<loophole::signal::ipc::IpcTarget> ParameterDomain::envelopeTargetForOrigin(
    loophole::signal::ipc::IpcOrigin origin
) const {
    using namespace loophole::signal::ipc;

    switch (origin) {
    case IpcOrigin::Aura:
        return IpcTarget::Aura;
    case IpcOrigin::Pulse:
        return IpcTarget::Pulse;
    case IpcOrigin::Signal:
        return IpcTarget::Signal;
    case IpcOrigin::Composer:
        return IpcTarget::Composer;
    }

    return std::nullopt;
}

void ParameterDomain::handleRequestDescriptors(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    const auto scope = parseScope(env.payload);
    if (!scope.has_value()) {
        emitError(
            env,
            session,
            "invalid_scope",
            "parameter.requestDescriptors requires scope.nodeId",
            nlohmann::json::object()
        );
        return;
    }

    PluginNode* pluginNode = resolvePluginNode(scope.value());
    if (!pluginNode || !pluginNode->getPlugin()) {
        emitError(
            env,
            session,
            "plugin_not_found",
            "Plugin node was not found for scope.nodeId",
            nlohmann::json{{"nodeId", scope->nodeId}}
        );
        return;
    }

    const auto target = envelopeTargetForOrigin(env.origin);
    if (!target.has_value()) {
        return;
    }

    IpcEnvelope event;
    event.version = 1;
    event.id = "parameter-descriptors-" + env.id;
    event.correlationId = env.id;
    event.timestamp = currentTimestamp();
    event.origin = IpcOrigin::Signal;
    event.target = target.value();
    event.domain = "parameter";
    event.kind = IpcKind::Event;
    event.name = "descriptorsSnapshot";
    event.priority = env.priority;

    nlohmann::json payload = nlohmann::json::object();
    payload["scope"] = makeScopePayload(scope->nodeId, scope->pluginInstanceId);

    nlohmann::json descriptors = nlohmann::json::array();
    for (const auto& descriptor : pluginNode->getPlugin()->listParameterDescriptors()) {
        descriptors.push_back({
            {"paramId", descriptor.paramId},
            {"name", descriptor.name},
            {"unit", descriptor.unit},
            {"min", descriptor.minValue},
            {"max", descriptor.maxValue},
            {"default", descriptor.defaultValue},
            {"step", descriptor.step},
            {"isAutomatable", descriptor.isAutomatable},
            {"isBypass", descriptor.isBypass}
        });
    }
    payload["descriptors"] = descriptors;

    event.payload = payload;
    session->send(event);
}

void ParameterDomain::handleRequestValues(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    const auto scope = parseScope(env.payload);
    if (!scope.has_value()) {
        emitError(
            env,
            session,
            "invalid_scope",
            "parameter.requestValues requires scope.nodeId",
            nlohmann::json::object()
        );
        return;
    }

    PluginNode* pluginNode = resolvePluginNode(scope.value());
    if (!pluginNode || !pluginNode->getPlugin()) {
        emitError(
            env,
            session,
            "plugin_not_found",
            "Plugin node was not found for scope.nodeId",
            nlohmann::json{{"nodeId", scope->nodeId}}
        );
        return;
    }

    std::vector<std::string> requestedParamIds;
    if (env.payload.contains("paramIds") && env.payload["paramIds"].is_array()) {
        for (const auto& entry : env.payload["paramIds"]) {
            if (entry.is_string()) {
                requestedParamIds.push_back(entry.get<std::string>());
            }
        }
    }

    const auto descriptors = pluginNode->getPlugin()->listParameterDescriptors();
    if (requestedParamIds.empty()) {
        requestedParamIds.reserve(descriptors.size());
        for (const auto& descriptor : descriptors) {
            requestedParamIds.push_back(descriptor.paramId);
        }
    }

    nlohmann::json values = nlohmann::json::object();
    for (const auto& paramId : requestedParamIds) {
        values[paramId] = pluginNode->getPlugin()->getParameterValue(paramId);
    }

    const auto target = envelopeTargetForOrigin(env.origin);
    if (!target.has_value()) {
        return;
    }

    IpcEnvelope event;
    event.version = 1;
    event.id = "parameter-values-" + env.id;
    event.correlationId = env.id;
    event.timestamp = currentTimestamp();
    event.origin = IpcOrigin::Signal;
    event.target = target.value();
    event.domain = "parameter";
    event.kind = IpcKind::Event;
    event.name = "valuesSnapshot";
    event.priority = env.priority;
    event.payload = {
        {"scope", makeScopePayload(scope->nodeId, scope->pluginInstanceId)},
        {"values", values}
    };

    session->send(event);
}

void ParameterDomain::handleSetValue(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    const auto scope = parseScope(env.payload);
    if (!scope.has_value()) {
        emitError(
            env,
            session,
            "invalid_scope",
            "parameter.setValue requires scope.nodeId",
            nlohmann::json::object()
        );
        return;
    }

    if (!env.payload.contains("paramId") || !env.payload["paramId"].is_string()) {
        emitError(
            env,
            session,
            "invalid_param_id",
            "parameter.setValue requires paramId",
            nlohmann::json::object()
        );
        return;
    }

    if (!env.payload.contains("value") || !env.payload["value"].is_number()) {
        emitError(
            env,
            session,
            "invalid_value",
            "parameter.setValue requires numeric value",
            nlohmann::json::object()
        );
        return;
    }

    PluginNode* pluginNode = resolvePluginNode(scope.value());
    if (!pluginNode || !pluginNode->getPlugin()) {
        emitError(
            env,
            session,
            "plugin_not_found",
            "Plugin node was not found for scope.nodeId",
            nlohmann::json{{"nodeId", scope->nodeId}}
        );
        return;
    }

    const std::string paramId = env.payload["paramId"].get<std::string>();
    const float requestedValue = env.payload["value"].get<float>();

    const auto descriptors = pluginNode->getPlugin()->listParameterDescriptors();
    const auto descriptorIt = std::find_if(
        descriptors.begin(),
        descriptors.end(),
        [&paramId](const PluginParameterDescriptor& descriptor) {
            return descriptor.paramId == paramId;
        }
    );

    if (descriptorIt == descriptors.end()) {
        emitError(
            env,
            session,
            "unknown_parameter",
            "parameter.setValue referenced an unknown paramId",
            nlohmann::json{{"paramId", paramId}}
        );
        return;
    }

    pluginNode->getPlugin()->setParameterValue(paramId, requestedValue);
    const float confirmedValue = pluginNode->getPlugin()->getParameterValue(paramId);

    const auto target = envelopeTargetForOrigin(env.origin);
    if (!target.has_value()) {
        return;
    }

    IpcEnvelope event;
    event.version = 1;
    event.id = "parameter-changed-" + env.id;
    event.correlationId = env.id;
    event.timestamp = currentTimestamp();
    event.origin = IpcOrigin::Signal;
    event.target = target.value();
    event.domain = "parameter";
    event.kind = IpcKind::Event;
    event.name = "valueChanged";
    event.priority = env.priority;
    event.payload = {
        {"scope", makeScopePayload(scope->nodeId, scope->pluginInstanceId)},
        {"paramId", paramId},
        {"value", confirmedValue}
    };

    session->send(event);
}

void ParameterDomain::emitError(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
    const std::string& code,
    const std::string& message,
    const nlohmann::json& details
) const {
    using namespace loophole::signal::ipc;

    const auto target = envelopeTargetForOrigin(env.origin);
    if (!target.has_value()) {
        return;
    }

    IpcEnvelope error;
    error.version = 1;
    error.id = "parameter-error-" + env.id;
    error.correlationId = env.id;
    error.timestamp = currentTimestamp();
    error.origin = IpcOrigin::Signal;
    error.target = target.value();
    error.domain = "parameter";
    error.kind = IpcKind::Error;
    error.name = env.name;
    error.priority = env.priority;
    error.payload = nlohmann::json::object();
    error.error = IpcErrorInfo{
        .code = code,
        .message = message,
        .details = details,
    };

    session->send(error);
}
