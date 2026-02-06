#include "domains/PluginDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/PluginHost.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include <nlohmann/json.hpp>

PluginDomain::PluginDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
}

void PluginDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    if (env.domain != "plugin") {
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
            "EngineHost is unavailable"
        );
        return;
    }

    if (env.name == "list") {
        handleList(env, session);
        return;
    }

    emitError(
        env,
        session,
        "unknown_command",
        "Unknown plugin command"
    );
}

std::optional<loophole::signal::ipc::IpcTarget> PluginDomain::envelopeTargetForOrigin(
    loophole::signal::ipc::IpcOrigin origin
) {
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

const char* PluginDomain::formatTag(PluginFormat format) noexcept {
    switch (format) {
    case PluginFormat::Clap:
        return "clap";
    case PluginFormat::Vst3:
        return "vst3";
    case PluginFormat::Au:
        return "au";
    case PluginFormat::Lv2:
        return "lv2";
    case PluginFormat::Native:
        return "native";
    }

    return "unknown";
}

void PluginDomain::handleList(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    auto target = envelopeTargetForOrigin(commandEnv.origin);
    if (!target.has_value()) {
        return;
    }

    auto* pluginHost = _engineHost->pluginHost();
    if (!pluginHost) {
        emitError(
            commandEnv,
            session,
            "plugin_host_unavailable",
            "Plugin host is unavailable"
        );
        return;
    }

    const auto descriptors = pluginHost->listPlugins();

    IpcEnvelope response;
    response.version = 1;
    response.id = "plugin-list-" + commandEnv.id;
    response.correlationId = commandEnv.id;
    response.timestamp = currentTimestamp();
    response.origin = IpcOrigin::Signal;
    response.target = target.value();
    response.domain = "plugin";
    response.kind = IpcKind::Event;
    response.name = "list";
    response.priority = commandEnv.priority;

    nlohmann::json plugins = nlohmann::json::array();
    for (const auto& descriptor : descriptors) {
        plugins.push_back({
            {"pluginId", descriptor.id},
            {"format", formatTag(descriptor.format)},
            {"displayName", descriptor.name},
            {"manufacturer", nullptr},
        });
    }

    response.payload = {
        {"plugins", plugins},
    };
    response.error = std::nullopt;

    session->send(response);
}

void PluginDomain::emitError(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
    const std::string& code,
    const std::string& message
) const {
    using namespace loophole::signal::ipc;

    auto target = envelopeTargetForOrigin(commandEnv.origin);
    if (!target.has_value()) {
        return;
    }

    IpcEnvelope errorEnv;
    errorEnv.version = 1;
    errorEnv.id = "plugin-error-" + commandEnv.id;
    errorEnv.correlationId = commandEnv.id;
    errorEnv.timestamp = currentTimestamp();
    errorEnv.origin = IpcOrigin::Signal;
    errorEnv.target = target.value();
    errorEnv.domain = "plugin";
    errorEnv.kind = IpcKind::Error;
    errorEnv.name = commandEnv.name;
    errorEnv.priority = commandEnv.priority;
    errorEnv.payload = nlohmann::json::object();
    IpcErrorInfo errorInfo;
    errorInfo.code = code;
    errorInfo.message = message;
    errorInfo.details = nlohmann::json::object();
    errorEnv.error = errorInfo;

    session->send(errorEnv);
}
