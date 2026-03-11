#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {
std::optional<std::vector<std::uint8_t>> encodeHardwareState(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter w;
        w.writeU32(1, 1);

        if (payload.contains("outputDevices") && payload["outputDevices"].is_array()) {
            std::vector<std::vector<std::uint8_t>> elements;
            for (const auto& dev : payload["outputDevices"]) {
                if (!dev.is_object()) {
                    continue;
                }

                TlvWriter dw;
                if (dev.contains("id") && dev["id"].is_string()) {
                    dw.writeString(2, dev["id"].get<std::string>());
                }
                if (dev.contains("name") && dev["name"].is_string()) {
                    dw.writeString(3, dev["name"].get<std::string>());
                }
                if (dev.contains("isDefault") && dev["isDefault"].is_boolean()) {
                    dw.writeBool(4, dev["isDefault"].get<bool>());
                }
                if (dev.contains("isActive") && dev["isActive"].is_boolean()) {
                    dw.writeBool(5, dev["isActive"].get<bool>());
                }
                if (dev.contains("maxChannels") && dev["maxChannels"].is_number_unsigned()) {
                    dw.writeU32(6, dev["maxChannels"].get<std::uint32_t>());
                }
                if (dev.contains("preferredSampleRate") && dev["preferredSampleRate"].is_number_unsigned()) {
                    dw.writeU32(7, dev["preferredSampleRate"].get<std::uint32_t>());
                }
                elements.push_back(dw.intoBytes());
            }

            if (!elements.empty()) {
                w.writeObjectList(2, elements);
            }
        }

        if (payload.contains("activeDeviceId") && payload["activeDeviceId"].is_string()) {
            w.writeString(3, payload["activeDeviceId"].get<std::string>());
        }
        if (payload.contains("preferredDeviceId") && payload["preferredDeviceId"].is_string()) {
            w.writeString(4, payload["preferredDeviceId"].get<std::string>());
        }
        if (payload.contains("lastError") && payload["lastError"].is_string()) {
            w.writeString(5, payload["lastError"].get<std::string>());
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}
} // namespace loophole::signal::ipc::binary_envelope
