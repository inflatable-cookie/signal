#include "domains/TransportDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/TransportState.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

TransportDomain::TransportDomain(EngineHost* engineHost) : _engineHost(engineHost) {
}

void TransportDomain::handle(const Envelope& env) {
    if (env.kind != "command") {
        std::cout << "[TransportDomain] Ignoring non-command: " << env.kind << std::endl;
        return;
    }

    if (!_engineHost) {
        std::cerr << "[TransportDomain] EngineHost is null" << std::endl;
        return;
    }

    auto& transport = _engineHost->transport();

    if (env.name == "play") {
        transport.isPlaying = true;
        std::cout << "[TransportDomain] Play command received" << std::endl;
    } else if (env.name == "stop") {
        transport.isPlaying = false;
        std::cout << "[TransportDomain] Stop command received" << std::endl;
    } else if (env.name == "seek") {
        try {
            nlohmann::json payload = nlohmann::json::parse(env.payload);
            if (payload.contains("seconds")) {
                transport.positionSeconds = payload["seconds"].get<double>();
            } else if (payload.contains("positionBeats")) {
                // For now, just use seconds - full beats support later
                double beats = payload["positionBeats"].get<double>();
                transport.positionSeconds = beats * 60.0 / 120.0; // Assume 120 BPM for now
            }
            std::cout << "[TransportDomain] Seek command received, position: "
                      << transport.positionSeconds << "s" << std::endl;
        } catch (const std::exception& e) {
            std::cerr << "[TransportDomain] Failed to parse seek payload: " << e.what() << std::endl;
        }
    } else if (env.name == "setLoopEnabled") {
        try {
            nlohmann::json payload = nlohmann::json::parse(env.payload);
            if (payload.contains("enabled")) {
                transport.loopEnabled = payload["enabled"].get<bool>();
                std::cout << "[TransportDomain] Loop enabled: " << transport.loopEnabled << std::endl;
            }
        } catch (const std::exception& e) {
            std::cerr << "[TransportDomain] Failed to parse setLoopEnabled payload: " << e.what() << std::endl;
        }
    } else if (env.name == "setLoopRegion") {
        try {
            nlohmann::json payload = nlohmann::json::parse(env.payload);
            if (payload.contains("startSeconds") && payload.contains("endSeconds")) {
                LoopRegion region;
                region.startSeconds = payload["startSeconds"].get<double>();
                region.endSeconds = payload["endSeconds"].get<double>();
                transport.loopRegion = region;
                std::cout << "[TransportDomain] Loop region set: " << region.startSeconds
                          << " - " << region.endSeconds << "s" << std::endl;
            } else if (payload.contains("startBeats") && payload.contains("endBeats")) {
                // Convert beats to seconds (assume 120 BPM for now)
                LoopRegion region;
                double startBeats = payload["startBeats"].get<double>();
                double endBeats = payload["endBeats"].get<double>();
                region.startSeconds = startBeats * 60.0 / 120.0;
                region.endSeconds = endBeats * 60.0 / 120.0;
                transport.loopRegion = region;
                std::cout << "[TransportDomain] Loop region set: " << region.startSeconds
                          << " - " << region.endSeconds << "s" << std::endl;
            }
        } catch (const std::exception& e) {
            std::cerr << "[TransportDomain] Failed to parse setLoopRegion payload: " << e.what() << std::endl;
        }
    } else {
        std::cout << "[TransportDomain] Unknown command: " << env.name << std::endl;
    }
}

