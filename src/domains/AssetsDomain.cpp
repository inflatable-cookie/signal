#include "domains/AssetsDomain.hpp"
#include "core/EngineHost.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

AssetsDomain::AssetsDomain(EngineHost* engineHost) : _engineHost(engineHost) {
}

void AssetsDomain::handle(const Envelope& env) {
    if (env.kind != "command") {
        std::cout << "[AssetsDomain] Ignoring non-command: " << env.kind << std::endl;
        return;
    }

    if (!_engineHost) {
        std::cerr << "[AssetsDomain] EngineHost is null" << std::endl;
        return;
    }

    if (env.name == "registerAudioAsset") {
        // Handle asset registration from Pulse
        try {
            nlohmann::json payload = env.payload;

            std::string assetId = payload.value("assetId", "");
            if (assetId.empty()) {
                std::cerr << "[AssetsDomain] Missing assetId in registerAudioAsset" << std::endl;
                return;
            }

            std::string path = payload.value("path", "");
            if (path.empty()) {
                std::cerr << "[AssetsDomain] Missing path in registerAudioAsset" << std::endl;
                return;
            }

            uint32_t channels = payload.value("channels", 2u);
            uint32_t sampleRate = payload.value("sampleRate", 44100u);
            uint64_t frames = payload.value("frames", 0u);

            // Diagnostic logging: asset registration
            std::cout << "[AssetsDomain] Registering audio asset: id='" << assetId
                      << "', path='" << path
                      << "', channels=" << channels
                      << ", sampleRate=" << sampleRate
                      << ", frames=" << frames << std::endl;

            if (frames == 0) {
                std::cerr << "[AssetsDomain] ⚠ Invalid frames count (0) for asset: " << assetId << std::endl;
            }

            _engineHost->prepareAudioAsset(assetId, path, channels, sampleRate, frames);
            std::cout << "[AssetsDomain] Called prepareAudioAsset for: " << assetId << std::endl;
        } catch (const std::exception& e) {
            std::cerr << "[AssetsDomain] Failed to parse registerAudioAsset payload: " << e.what() << std::endl;
        }
    } else {
        std::cout << "[AssetsDomain] Unknown command: " << env.name << std::endl;
    }
}

