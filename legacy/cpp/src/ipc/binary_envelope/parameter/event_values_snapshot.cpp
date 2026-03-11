#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<std::vector<std::uint8_t>> encodeParameterValuesSnapshot(
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

        if (payload.contains("values") && payload["values"].is_object()) {
            std::vector<std::vector<std::uint8_t>> valueObjects;
            for (auto it = payload["values"].begin(); it != payload["values"].end(); ++it) {
                if (!it.value().is_number()) {
                    continue;
                }

                TlvWriter vw;
                vw.writeU32(1, 1);
                vw.writeString(2, it.key());
                vw.writeF64(3, it.value().get<double>());
                valueObjects.push_back(vw.intoBytes());
            }

            if (!valueObjects.empty()) {
                w.writeObjectList(3, valueObjects);
            }
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc::binary_envelope
