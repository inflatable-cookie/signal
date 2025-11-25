#include "domains/AutomationDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/AutomationService.hpp"
#include "core/AutomationData.hpp"
#include "core/ScheduleData.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>
#include <nlohmann/json.hpp>
#include <cmath>

AutomationDomain::AutomationDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
    std::cout << "[AutomationDomain] Initialised" << std::endl;
}

void AutomationDomain::handle(const Envelope& env) {
    if (env.domain != "automation" || env.kind != "command") {
        return;
    }

    if (env.name == "setCurvesForSession") {
        try {
            nlohmann::json payload = nlohmann::json::parse(env.payload);
            std::vector<AutomationCurve> curves;

            if (payload.contains("curves") && payload["curves"].is_array()) {
                for (const auto& curveJson : payload["curves"]) {
                    AutomationCurve curve;
                    curve.targetId = curveJson["targetId"];
                    curve.parameter = curveJson["parameter"];

                    if (curveJson.contains("points") && curveJson["points"].is_array()) {
                        for (const auto& pointJson : curveJson["points"]) {
                            AutomationCurvePoint point;
                            point.timeSamples = pointJson["timeSamples"];
                            point.value = pointJson["value"];
                            point.shape = pointJson.value("shape", "linear"); // Default to linear if not present
                            curve.points.push_back(point);
                        }
                    }

                    curves.push_back(curve);
                }
            }

            _engineHost->automation().setCurvesForSession(curves);

            std::cout << "[AutomationDomain] Set " << curves.size() << " automation curves" << std::endl;
        } catch (const std::exception& e) {
            std::cerr << "[AutomationDomain] Failed to parse setCurvesForSession payload: " << e.what() << std::endl;
        }
    } else if (env.name == "automationSnapshot") {
        // Handle AutomationSnapshot from Pulse
        // Architecture: Pulse sends AutomationSnapshot with automation events
        // Signal converts beats to samples and applies automation in renderBlock
        try {
            nlohmann::json payload = env.payload;
            double sampleRate = _engineHost->getSampleRate();

            AutomationData automationData;

            // Parse tempo map
            if (payload.contains("tempoMap")) {
                const auto& tempoMapJson = payload["tempoMap"];
                automationData.tempoMap.defaultTempo = tempoMapJson.value("bpm", 120.0);
                // TODO: Parse tempo map entries if present
            } else {
                automationData.tempoMap.defaultTempo = 120.0;
            }

            // Parse automation events
            if (payload.contains("events") && payload["events"].is_array()) {
                for (const auto& eventJson : payload["events"]) {
                    AutomationEventCompiled event;
                    event.nodeId = eventJson.value("nodeId", "");
                    event.paramId = eventJson.value("paramId", "");
                    event.valueNorm = eventJson.value("valueNorm", 0.0f);

                    // Convert curve string to enum
                    std::string curveStr = eventJson.value("curve", "linear");
                    if (curveStr == "step") {
                        event.curve = AutomationCurveType::Step;
                    } else {
                        event.curve = AutomationCurveType::Linear;
                    }

                    // Convert timeBeats to timeSamples
                    double timeBeats = eventJson.value("timeBeats", 0.0);
                    // Simple conversion: beats to samples = (beats / tempo) * sampleRate * 60
                    double tempo = automationData.tempoMap.defaultTempo;
                    event.timeSamples = static_cast<uint64_t>((timeBeats / tempo) * sampleRate * 60.0);

                    automationData.events.push_back(event);
                }
            }

            // Sort events by timeSamples
            std::sort(automationData.events.begin(), automationData.events.end(),
                [](const AutomationEventCompiled& a, const AutomationEventCompiled& b) {
                    return a.timeSamples < b.timeSamples;
                });

            // Load automation snapshot into EngineHost
            _engineHost->loadAutomationSnapshot(automationData);

            std::cout << "[AutomationDomain] Loaded automation snapshot: "
                      << automationData.events.size() << " events" << std::endl;
        } catch (const std::exception& e) {
            std::cerr << "[AutomationDomain] Failed to parse automationSnapshot payload: " << e.what() << std::endl;
        }
    } else if (env.name == "updateCurve") {
        try {
            nlohmann::json payload = nlohmann::json::parse(env.payload);
            AutomationCurve curve;
            curve.targetId = payload["targetId"];
            curve.parameter = payload["parameter"];

            if (payload.contains("points") && payload["points"].is_array()) {
                for (const auto& pointJson : payload["points"]) {
                    AutomationCurvePoint point;
                    point.timeSamples = pointJson["timeSamples"];
                    point.value = pointJson["value"];
                    point.shape = pointJson.value("shape", "linear"); // Default to linear if not present
                    curve.points.push_back(point);
                }
            }

            _engineHost->automation().updateCurve(curve);

            std::cout << "[AutomationDomain] Updated curve for " << curve.targetId << "." << curve.parameter << std::endl;
        } catch (const std::exception& e) {
            std::cerr << "[AutomationDomain] Failed to parse updateCurve payload: " << e.what() << std::endl;
        }
    } else {
        std::cout << "[AutomationDomain] Received unhandled automation command: " << env.name << std::endl;
    }
}

