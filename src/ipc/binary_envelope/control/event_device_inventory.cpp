#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<std::vector<std::uint8_t>> encodeControlDeviceInventory(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter w;
        w.writeU32(1, 1);

        if (payload.contains("devices") && payload["devices"].is_array()) {
            std::vector<std::vector<std::uint8_t>> elements;
            for (const auto& dev : payload["devices"]) {
                if (!dev.is_object()) {
                    continue;
                }

                TlvWriter dw;
                if (dev.contains("id") && dev["id"].is_string()) {
                    dw.writeString(2, dev["id"].get<std::string>());
                }
                if (dev.contains("kind") && dev["kind"].is_string()) {
                    dw.writeString(3, dev["kind"].get<std::string>());
                }
                if (dev.contains("name") && dev["name"].is_string()) {
                    dw.writeString(4, dev["name"].get<std::string>());
                }
                if (dev.contains("manufacturer") && dev["manufacturer"].is_string()) {
                    dw.writeString(5, dev["manufacturer"].get<std::string>());
                }
                if (dev.contains("connectionState") && dev["connectionState"].is_string()) {
                    dw.writeString(6, dev["connectionState"].get<std::string>());
                }
                elements.push_back(dw.intoBytes());
            }

            if (!elements.empty()) {
                w.writeObjectList(2, elements);
            }
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc::binary_envelope
