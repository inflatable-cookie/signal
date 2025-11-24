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

    double sampleRate = _engineHost->getSampleRate();
    double tempo = transport.tempo; // Get tempo from transport state

    if (env.name == "play") {
        transport.isPlaying = true;
        // Update playhead from transport position (in case we seeked while stopped)
        uint64_t playheadSamples = static_cast<uint64_t>(transport.positionSeconds * sampleRate);
        _engineHost->setPlayheadSamples(playheadSamples);
        _engineHost->commitTransportUpdate();  // Commit snapshot swap
        std::cout << "[TransportDomain] Play command received, playhead: " << playheadSamples << " samples" << std::endl;
    } else if (env.name == "stop") {
        transport.isPlaying = false;
        // Update transport position from playhead when stopping
        uint64_t playheadSamples = _engineHost->getPlayheadSamples();
        transport.positionSeconds = static_cast<double>(playheadSamples) / sampleRate;
        _engineHost->commitTransportUpdate();  // Commit snapshot swap
        std::cout << "[TransportDomain] Stop command received, position: " << transport.positionSeconds << "s" << std::endl;
    } else if (env.name == "seek") {
        try {
            nlohmann::json payload = nlohmann::json::parse(env.payload);
            double positionSeconds = 0.0;
            if (payload.contains("positionSamples")) {
                // Direct sample position (preferred)
                uint64_t samples = payload["positionSamples"].get<uint64_t>();
                positionSeconds = static_cast<double>(samples) / sampleRate; // Use engine sample rate
            } else if (payload.contains("seconds")) {
                positionSeconds = payload["seconds"].get<double>();
            } else if (payload.contains("positionBeats")) {
                // Convert beats to seconds using real tempo
                double beats = payload["positionBeats"].get<double>();
                positionSeconds = (beats / tempo) * 60.0;
            }
            transport.positionSeconds = positionSeconds;

            // Update playhead in samples
            uint64_t playheadSamples = static_cast<uint64_t>(positionSeconds * sampleRate);
            _engineHost->setPlayheadSamples(playheadSamples);
            _engineHost->commitTransportUpdate();  // Commit snapshot swap

            std::cout << "[TransportDomain] Seek command received, position: "
                      << transport.positionSeconds << "s (" << playheadSamples << " samples)" << std::endl;
        } catch (const std::exception& e) {
            std::cerr << "[TransportDomain] Failed to parse seek payload: " << e.what() << std::endl;
        }
    } else if (env.name == "setLoopEnabled") {
        try {
            nlohmann::json payload = nlohmann::json::parse(env.payload);
            if (payload.contains("enabled")) {
                transport.loopEnabled = payload["enabled"].get<bool>();
                _engineHost->commitTransportUpdate();  // Commit snapshot swap
                std::cout << "[TransportDomain] Loop enabled: " << transport.loopEnabled << std::endl;
            }
        } catch (const std::exception& e) {
            std::cerr << "[TransportDomain] Failed to parse setLoopEnabled payload: " << e.what() << std::endl;
        }
    } else if (env.name == "setLoopRegion") {
        try {
            nlohmann::json payload = nlohmann::json::parse(env.payload);
            LoopRegion region;
            bool hasRegion = false;

            // Prefer samples (per spec), fall back to seconds or beats
            if (payload.contains("startSamples") && payload.contains("endSamples")) {
                uint64_t startSamples = payload["startSamples"].get<uint64_t>();
                uint64_t endSamples = payload["endSamples"].get<uint64_t>();
                region.startSeconds = static_cast<double>(startSamples) / sampleRate;
                region.endSeconds = static_cast<double>(endSamples) / sampleRate;
                hasRegion = true;
            } else if (payload.contains("startSeconds") && payload.contains("endSeconds")) {
                region.startSeconds = payload["startSeconds"].get<double>();
                region.endSeconds = payload["endSeconds"].get<double>();
                hasRegion = true;
            } else if (payload.contains("startBeats") && payload.contains("endBeats")) {
                // Convert beats to seconds using real tempo
                double startBeats = payload["startBeats"].get<double>();
                double endBeats = payload["endBeats"].get<double>();
                region.startSeconds = (startBeats / tempo) * 60.0;
                region.endSeconds = (endBeats / tempo) * 60.0;
                hasRegion = true;
            }

            if (hasRegion) {
                transport.loopRegion = region;
                _engineHost->commitTransportUpdate();  // Commit snapshot swap
                std::cout << "[TransportDomain] Loop region set: " << region.startSeconds
                          << " - " << region.endSeconds << "s" << std::endl;
            } else {
                // Clear loop region if enabled is false
                if (payload.contains("enabled") && !payload["enabled"].get<bool>()) {
                    transport.loopRegion = std::nullopt;
                    _engineHost->commitTransportUpdate();  // Commit snapshot swap
                    std::cout << "[TransportDomain] Loop region cleared" << std::endl;
                }
            }
        } catch (const std::exception& e) {
            std::cerr << "[TransportDomain] Failed to parse setLoopRegion payload: " << e.what() << std::endl;
        }
    } else if (env.name == "setTempo") {
        try {
            nlohmann::json payload = nlohmann::json::parse(env.payload);
            if (payload.contains("tempo")) {
                transport.tempo = payload["tempo"].get<double>();
                _engineHost->commitTransportUpdate();  // Commit snapshot swap
                std::cout << "[TransportDomain] Tempo set to: " << transport.tempo << " BPM" << std::endl;
            }
        } catch (const std::exception& e) {
            std::cerr << "[TransportDomain] Failed to parse setTempo payload: " << e.what() << std::endl;
        }
    } else {
        std::cout << "[TransportDomain] Unknown command: " << env.name << std::endl;
    }
}

