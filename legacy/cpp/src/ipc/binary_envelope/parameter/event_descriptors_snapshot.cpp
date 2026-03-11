#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<std::vector<std::uint8_t>> encodeParameterDescriptorsSnapshot(
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

        if (payload.contains("descriptors") && payload["descriptors"].is_array()) {
            std::vector<std::vector<std::uint8_t>> descriptorObjects;
            for (const auto& descriptor : payload["descriptors"]) {
                if (!descriptor.is_object()) {
                    continue;
                }

                TlvWriter dw;
                dw.writeU32(1, 1);

                if (descriptor.contains("paramId") && descriptor["paramId"].is_string()) {
                    dw.writeString(2, descriptor["paramId"].get<std::string>());
                }
                if (descriptor.contains("name") && descriptor["name"].is_string()) {
                    dw.writeString(3, descriptor["name"].get<std::string>());
                }
                if (descriptor.contains("unit") && descriptor["unit"].is_string()) {
                    dw.writeString(4, descriptor["unit"].get<std::string>());
                }
                if (descriptor.contains("min") && descriptor["min"].is_number()) {
                    dw.writeF64(5, descriptor["min"].get<double>());
                }
                if (descriptor.contains("max") && descriptor["max"].is_number()) {
                    dw.writeF64(6, descriptor["max"].get<double>());
                }
                if (descriptor.contains("default") && descriptor["default"].is_number()) {
                    dw.writeF64(7, descriptor["default"].get<double>());
                }
                if (descriptor.contains("step") && descriptor["step"].is_number()) {
                    dw.writeF64(8, descriptor["step"].get<double>());
                }
                if (descriptor.contains("isAutomatable") && descriptor["isAutomatable"].is_boolean()) {
                    dw.writeBool(9, descriptor["isAutomatable"].get<bool>());
                }
                if (descriptor.contains("isBypass") && descriptor["isBypass"].is_boolean()) {
                    dw.writeBool(10, descriptor["isBypass"].get<bool>());
                }

                descriptorObjects.push_back(dw.intoBytes());
            }

            if (!descriptorObjects.empty()) {
                w.writeObjectList(3, descriptorObjects);
            }
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc::binary_envelope
