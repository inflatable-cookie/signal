#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<std::vector<std::uint8_t>> encodeParameterValueChanged(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter w;
        w.writeU32(1, 1);

        if (payload.contains("scope") && payload["scope"].is_object()) {
            const auto& scope = payload["scope"];
            w.writeObject(2, [&scope](TlvWriter& sw) {
                sw.writeU32(1, 1);

                if (scope.contains("nodeId") && scope["nodeId"].is_string()) {
                    sw.writeString(2, scope["nodeId"].get<std::string>());
                }

                if (scope.contains("pluginInstanceId") && scope["pluginInstanceId"].is_string()) {
                    sw.writeString(3, scope["pluginInstanceId"].get<std::string>());
                }
            });
        }

        if (payload.contains("paramId") && payload["paramId"].is_string()) {
            w.writeString(3, payload["paramId"].get<std::string>());
        }

        if (payload.contains("value") && payload["value"].is_number()) {
            w.writeF64(4, payload["value"].get<double>());
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc::binary_envelope
