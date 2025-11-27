#include "domains/MixerDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/MixerService.hpp"
#include "core/GraphEngine.hpp"
#include "core/GraphNodes.hpp"
#include "core/GraphNode.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

MixerDomain::MixerDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
    std::cout << "[MixerDomain] Initialised" << std::endl;
}

void MixerDomain::handle(const Envelope& env) {
    if (env.domain != "mixer" || env.kind != "command") {
        return;
    }

    if (env.name == "updateChannel") {
        try {
            nlohmann::json payload = nlohmann::json::parse(env.payload);
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
                        std::cout << "[MixerDomain] Applied gain=" << effectiveGain
                                  << " pan=" << pan
                                  << " to MixerChannelNode " << nodeId << std::endl;
                    }
                } else {
                    std::cout << "[MixerDomain] Warning: MixerChannelNode not found for trackId=" << trackId
                              << " (nodeId=" << nodeId << ")" << std::endl;
                }
            }

            std::cout << "[MixerDomain] Updated channel " << channelId
                      << " gain=" << gain
                      << " pan=" << pan
                      << " muted=" << isMuted
                      << " soloed=" << isSoloed
                      << " effectiveMuted=" << effectiveMuted << std::endl;
        } catch (const std::exception& e) {
            std::cerr << "[MixerDomain] Failed to parse updateChannel payload: " << e.what() << std::endl;
        }
    } else {
        std::cout << "[MixerDomain] Received unhandled mixer command: " << env.name << std::endl;
    }
}

