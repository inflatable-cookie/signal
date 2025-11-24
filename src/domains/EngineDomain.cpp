#include "domains/EngineDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/ClipScheduler.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

EngineDomain::EngineDomain(EngineHost* engineHost) : _engineHost(engineHost) {
}

void EngineDomain::handle(const Envelope& env) {
    if (env.kind != "command") {
        std::cout << "[EngineDomain] Ignoring non-command: " << env.kind << std::endl;
        return;
    }

    if (!_engineHost) {
        std::cerr << "[EngineDomain] EngineHost is null" << std::endl;
        return;
    }

    if (env.name == "start") {
        _engineHost->start();
    } else if (env.name == "stop") {
        _engineHost->stop();
    } else if (env.name == "reset") {
        _engineHost->reset();
    } else if (env.name == "shutdown") {
        std::cout << "[EngineDomain] Shutdown requested" << std::endl;
        _engineHost->shutdown();
    } else if (env.name == "heartbeat") {
        // Heartbeat command received - handled by DomainDispatcher to emit event
        std::cout << "[EngineDomain] Heartbeat command received" << std::endl;
    } else if (env.name == "scheduleSession") {
        // Handle schedule session command
        try {
            nlohmann::json payload = env.payload;
            std::vector<ScheduledClip> clips;

            if (payload.contains("clips") && payload["clips"].is_array()) {
                for (const auto& clipJson : payload["clips"]) {
                    ScheduledClip clip;
                    clip.clipId = clipJson.value("clipId", "");
                    clip.channelId = clipJson.value("channelId", "");
                    clip.startBeats = clipJson.value("startBeats", 0.0);
                    clip.durationBeats = clipJson.value("durationBeats", 0.0);
                    clip.gainDb = clipJson.value("gainDb", 0.0);
                    clip.muted = clipJson.value("muted", false);
                    clips.push_back(clip);
                }
            }

            // Get tempo from transport (default 120 BPM if not available)
            double tempo = 120.0; // TODO: Get from transport state or session
            double sampleRate = _engineHost->getSampleRate();

            // Apply schedule to ClipScheduler
            _engineHost->clipScheduler().setSchedule(clips, tempo, sampleRate);

            std::cout << "[EngineDomain] Applied schedule: " << clips.size() << " clips" << std::endl;
        } catch (const std::exception& e) {
            std::cerr << "[EngineDomain] Failed to parse scheduleSession payload: " << e.what() << std::endl;
        }
    } else {
        std::cout << "[EngineDomain] Unknown command: " << env.name << std::endl;
    }
}

