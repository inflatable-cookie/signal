#include "domains/AutomationDomain.hpp"
#include "core/EngineHost.hpp"
#include "core/AutomationService.hpp"
#include "ipc/Envelope.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

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
            nlohmann::json payload = env.payload;
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
    } else if (env.name == "updateCurve") {
        try {
            nlohmann::json payload = env.payload;
            AutomationCurve curve;
            curve.targetId = payload["targetId"];
            curve.parameter = payload["parameter"];

            if (payload.contains("points") && payload["points"].is_array()) {
                for (const auto& pointJson : payload["points"]) {
                    AutomationCurvePoint point;
                    point.timeSamples = pointJson["timeSamples"];
                    point.value = pointJson["value"];
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

