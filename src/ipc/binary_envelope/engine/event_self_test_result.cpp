#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"
#include <limits>

namespace loophole::signal::ipc::binary_envelope {
std::optional<std::vector<std::uint8_t>> encodeEngineSelfTestResult(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter w;
        w.writeU32(1, 1);

        if (payload.contains("ok") && payload["ok"].is_boolean()) {
            w.writeBool(2, payload["ok"].get<bool>());
        }

        auto toU32 = [](const nlohmann::json& v) -> std::optional<std::uint32_t> {
            if (v.is_number_unsigned()) {
                return v.get<std::uint32_t>();
            }
            if (v.is_number_integer()) {
                auto i = v.get<std::int64_t>();
                if (i >= 0 && i <= static_cast<std::int64_t>(std::numeric_limits<std::uint32_t>::max())) {
                    return static_cast<std::uint32_t>(i);
                }
                return std::nullopt;
            }
            return std::nullopt;
        };

        if (payload.contains("scenarioCount")) {
            auto v = toU32(payload["scenarioCount"]);
            if (v.has_value()) {
                w.writeU32(3, v.value());
            }
        }

        if (payload.contains("failedScenarioCount")) {
            auto v = toU32(payload["failedScenarioCount"]);
            if (v.has_value()) {
                w.writeU32(4, v.value());
            }
        }

        if (payload.contains("scenarios") && payload["scenarios"].is_array()) {
            std::vector<std::vector<std::uint8_t>> elements;
            for (const auto& sc : payload["scenarios"]) {
                if (!sc.is_object()) {
                    continue;
                }

                TlvWriter sw;
                if (sc.contains("id") && sc["id"].is_string()) {
                    sw.writeString(2, sc["id"].get<std::string>());
                }
                if (sc.contains("ok") && sc["ok"].is_boolean()) {
                    sw.writeBool(3, sc["ok"].get<bool>());
                }
                if (sc.contains("maxAbsSample") && sc["maxAbsSample"].is_number()) {
                    sw.writeF64(4, sc["maxAbsSample"].get<double>());
                }
                elements.push_back(sw.intoBytes());
            }

            if (!elements.empty()) {
                w.writeObjectList(5, elements);
            }
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}
} // namespace loophole::signal::ipc::binary_envelope
