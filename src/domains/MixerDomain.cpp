#include "domains/MixerDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/MixerService.hpp"
#include "core/GraphEngine.hpp"
#include "core/GraphNodes.hpp"
#include "core/GraphNode.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <sstream>

MixerDomain::MixerDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
    LOG_INFO({"MixerDomain"}, "Initialised");
}

void MixerDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    if (env.domain != "mixer") {
        LOG_DEBUG({"MixerDomain"}, "Received envelope for different domain");
        return;
    }

    if (env.kind != loophole::signal::ipc::IpcKind::Command) {
        LOG_DEBUG({"MixerDomain"}, "Ignoring non-command envelope");
        return;
    }

    if (env.name == "updateChannel") {
        handleUpdateChannel(env.payload);
    } else {
        LOG_WARN({"MixerDomain"}, std::string("Received unhandled mixer command: ") + env.name);
    }
}

void MixerDomain::handleUpdateChannel(const nlohmann::json& payload) {
    try {
        std::string channelId = payload["channelId"];
        float gain = payload["gain"];
        float pan = payload.value("pan", 0.0f); // Default to 0.0 if not present (backward compatibility)
        bool isMuted = payload["isMuted"];
        bool isSoloed = payload["isSoloed"];
        bool effectiveMuted = payload["effectiveMuted"];

        // Update MixerService state
        _engineHost->mixer().updateChannel(
            channelId,
            gain,
            pan,
            isMuted,
            isSoloed,
            effectiveMuted
        );

        // Apply gain to MixerChannelNode in graph (if trackId is provided)
        if (payload.contains("trackId") && payload["trackId"].is_string()) {
            std::string trackId = payload["trackId"];
            // MixerChannelNode ID format: "mixer-{trackId}"
            std::string nodeId = "mixer-" + trackId;
            GraphNode* node = _engineHost->graphEngine().findNode(nodeId);
            if (node && node->getKind() == NodeKind::MixerChannel) {
                auto* mixerNode = dynamic_cast<MixerChannelNode*>(node);
                if (mixerNode) {
                    // Apply effective gain (0.0 if muted, otherwise use gain value)
                    float effectiveGain = effectiveMuted ? 0.0f : gain;
                    mixerNode->setGain(effectiveGain);
                    mixerNode->setPan(pan);
                    std::ostringstream msg;
                    msg << "Applied gain=" << effectiveGain
                        << " pan=" << pan
                        << " to MixerChannelNode " << nodeId;
                    LOG_DEBUG({"MixerDomain"}, msg.str());
                }
            } else {
                std::ostringstream msg;
                msg << "Warning: MixerChannelNode not found for trackId=" << trackId
                    << " (nodeId=" << nodeId << ")";
                LOG_WARN({"MixerDomain"}, msg.str());
            }
        }

        std::ostringstream msg;
        msg << "Updated channel " << channelId
            << " gain=" << gain
            << " pan=" << pan
            << " muted=" << isMuted
            << " soloed=" << isSoloed
            << " effectiveMuted=" << effectiveMuted;
        LOG_DEBUG({"MixerDomain"}, msg.str());
    } catch (const std::exception& e) {
        LOG_ERROR({"MixerDomain"}, std::string("Failed to parse updateChannel payload: ") + e.what());
    }
}

