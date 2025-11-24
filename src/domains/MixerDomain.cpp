#include "domains/MixerDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/MixerService.hpp"
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
            nlohmann::json payload = env.payload;
            std::string channelId = payload["channelId"];
            float gain = payload["gain"];
            float pan = payload.value("pan", 0.0f); // Default to 0.0 if not present (backward compatibility)
            bool isMuted = payload["isMuted"];
            bool isSoloed = payload["isSoloed"];
            bool effectiveMuted = payload["effectiveMuted"];

            _engineHost->mixer().updateChannel(
                channelId,
                gain,
                pan,
                isMuted,
                isSoloed,
                effectiveMuted
            );

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

