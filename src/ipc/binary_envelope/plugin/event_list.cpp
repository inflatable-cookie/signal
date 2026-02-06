#include "ipc/binary_envelope/CodecCommon.hpp"

namespace loophole::signal::ipc::binary_envelope {
namespace {

std::vector<std::uint8_t> buildPluginDescriptor(
    const nlohmann::json& plugin
) {
    TlvWriter objectWriter;

    if (plugin.contains("pluginId") && plugin["pluginId"].is_string()) {
        objectWriter.writeString(2, plugin["pluginId"].get<std::string>());
    }

    if (plugin.contains("format") && plugin["format"].is_string()) {
        objectWriter.writeString(3, plugin["format"].get<std::string>());
    }

    if (plugin.contains("displayName") && plugin["displayName"].is_string()) {
        objectWriter.writeString(4, plugin["displayName"].get<std::string>());
    }

    if (plugin.contains("manufacturer") && plugin["manufacturer"].is_string()) {
        objectWriter.writeString(5, plugin["manufacturer"].get<std::string>());
    }

    return objectWriter.intoBytes();
}

} // namespace

std::optional<std::vector<std::uint8_t>> encodePluginListEvent(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter writer;
        writer.writeU32(1, 1);

        if (payload.contains("plugins") && payload["plugins"].is_array()) {
            std::vector<std::vector<std::uint8_t>> elements;
            for (const auto& plugin : payload["plugins"]) {
                if (!plugin.is_object()) {
                    continue;
                }
                elements.push_back(buildPluginDescriptor(plugin));
            }
            writer.writeObjectList(2, elements);
        }

        return writer.intoBytes();
    } catch (const std::exception& ex) {
        error = ex.what();
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc::binary_envelope
