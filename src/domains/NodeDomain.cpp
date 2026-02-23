#include "domains/NodeDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/GraphEngine.hpp"
#include "core/GraphNodes.hpp"
#include "core/GraphNode.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <sstream>

namespace {
std::optional<int> parseSpatialChannelGainIndex(const std::string& parameterId) {
    static const std::string prefix = "spatial.channelGain.";

    if (parameterId.rfind(prefix, 0) != 0) {
        return std::nullopt;
    }

    const std::string suffix = parameterId.substr(prefix.size());
    if (suffix.empty()) {
        return std::nullopt;
    }

    try {
        size_t pos = 0;
        int index = std::stoi(suffix, &pos, 10);
        if (pos != suffix.size()) {
            return std::nullopt;
        }
        if (index < 0) {
            return std::nullopt;
        }
        return index;
    } catch (...) {
        return std::nullopt;
    }
}
}

NodeDomain::NodeDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
    LOG_DEBUG({"NodeDomain"}, "Initialised");
}

void NodeDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    if (env.domain != "node") {
        LOG_DEBUG({"NodeDomain"}, "Received envelope for different domain");
        return;
    }

    if (env.kind != loophole::signal::ipc::IpcKind::Command) {
        LOG_DEBUG({"NodeDomain"}, "Ignoring non-command envelope");
        return;
    }

    if (env.name == "setParameter") {
        handleSetParameter(env.payload);
    } else {
        LOG_WARN({"NodeDomain"}, std::string("Received unhandled node command: ") + env.name);
    }
}

void NodeDomain::handleSetParameter(const nlohmann::json& payload) {
    try {
        const std::string nodeId = payload.value("nodeId", "");
        const std::string parameterId = payload.value("parameterId", "");

        if (nodeId.empty() || parameterId.empty()) {
            LOG_WARN({"NodeDomain"}, "setParameter payload missing nodeId or parameterId");
            return;
        }

        GraphNode* node = _engineHost->graphEngine().findNode(nodeId);

        if (!node) {
            std::ostringstream msg;
            msg << "Node not found for nodeId=" << nodeId << " in setParameter";
            LOG_WARN({"NodeDomain"}, msg.str());
            return;
        }

        if (node->getKind() == NodeKind::Fader) {
            auto* faderNode = dynamic_cast<FaderNode*>(node);

            if (!faderNode) {
                std::ostringstream msg;
                msg << "Node " << nodeId << " reported as Fader but dynamic_cast failed";
                LOG_WARN({"NodeDomain"}, msg.str());
                return;
            }

            if (parameterId == "gain") {
                const float value = payload.value("value", 0.0f);
                faderNode->setGain(value);
            } else if (parameterId == "spatial.balance") {
                const float value = payload.value("value", 0.0f);
                faderNode->setPan(value);
            } else if (auto channelIndex = parseSpatialChannelGainIndex(parameterId)) {
                const float value = payload.value("value", 1.0f);
                faderNode->setChannelGain(channelIndex.value(), value);
            } else if (parameterId == "muted") {
                if (!payload.contains("value") || !payload["value"].is_boolean()) {
                    std::ostringstream msg;
                    msg << "setParameter muted expects boolean value for nodeId=" << nodeId;
                    LOG_WARN({"NodeDomain"}, msg.str());
                    return;
                }
                faderNode->setMuted(payload["value"].get<bool>());
            } else {
                std::ostringstream msg;
                msg << "Unhandled Fader parameterId=" << parameterId
                    << " for nodeId=" << nodeId;
                LOG_WARN({"NodeDomain"}, msg.str());
            }

            std::ostringstream msg;
            msg << "Applied setParameter nodeId=" << nodeId
                << " parameterId=" << parameterId
                << " value=" << payload.value("value", 0.0f);
            LOG_DEBUG({"NodeDomain"}, msg.str());
        } else if (
            node->getKind() == NodeKind::Instrument
            || node->getKind() == NodeKind::AudioFx
            || node->getKind() == NodeKind::MidiFx
        ) {
            auto* pluginNode = dynamic_cast<PluginNode*>(node);
            if (!pluginNode) {
                std::ostringstream msg;
                msg << "Node " << nodeId << " reported as PluginNode kind but dynamic_cast failed";
                LOG_WARN({"NodeDomain"}, msg.str());
                return;
            }

            if (parameterId == "muted") {
                if (!payload.contains("value") || !payload["value"].is_boolean()) {
                    std::ostringstream msg;
                    msg << "setParameter muted expects boolean value for nodeId=" << nodeId;
                    LOG_WARN({"NodeDomain"}, msg.str());
                    return;
                }
                pluginNode->setMuted(payload["value"].get<bool>());
            } else {
                std::ostringstream msg;
                msg << "Unhandled PluginNode parameterId=" << parameterId
                    << " for nodeId=" << nodeId;
                LOG_WARN({"NodeDomain"}, msg.str());
            }
        } else {
            // Future: handle other node kinds once parameter routing is unified.
            std::ostringstream msg;
            msg << "setParameter for unsupported node kind on nodeId=" << nodeId;
            LOG_WARN({"NodeDomain"}, msg.str());
        }
    } catch (const std::exception& e) {
        LOG_ERROR({"NodeDomain"}, std::string("Failed to handle setParameter payload: ") + e.what());
    }
}
