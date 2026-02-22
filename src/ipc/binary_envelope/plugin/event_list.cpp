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

std::optional<std::vector<std::uint8_t>> encodeSchemaOnly(
    const nlohmann::json&,
    std::string&
) {
    TlvWriter writer;
    writer.writeU32(1, 1);
    return writer.intoBytes();
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

std::optional<std::vector<std::uint8_t>> encodePluginRescanEvent(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter writer;
        writer.writeU32(1, 1);
        if (payload.contains("scanId") && payload["scanId"].is_string()) {
            writer.writeString(4, payload["scanId"].get<std::string>());
        }
        if (payload.contains("scanLevel") && payload["scanLevel"].is_string()) {
            writer.writeString(5, payload["scanLevel"].get<std::string>());
        }
        return writer.intoBytes();
    } catch (const std::exception& ex) {
        error = ex.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodePluginScanStartedEvent(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter writer;
        writer.writeU32(1, 1);
        if (payload.contains("scanId") && payload["scanId"].is_string()) {
            writer.writeString(2, payload["scanId"].get<std::string>());
        }
        if (payload.contains("fullScan") && payload["fullScan"].is_boolean()) {
            writer.writeU32(3, payload["fullScan"].get<bool>() ? 1U : 0U);
        }
        if (payload.contains("scanLevel") && payload["scanLevel"].is_string()) {
            writer.writeString(4, payload["scanLevel"].get<std::string>());
        }
        return writer.intoBytes();
    } catch (const std::exception& ex) {
        error = ex.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodePluginAddedEvent(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter writer;
        writer.writeU32(1, 1);
        if (payload.contains("scanId") && payload["scanId"].is_string()) {
            writer.writeString(2, payload["scanId"].get<std::string>());
        }

        const auto* plugin = payload.contains("plugin") && payload["plugin"].is_object()
            ? &payload["plugin"]
            : nullptr;
        if (plugin != nullptr) {
            writer.writeObject(3, [&plugin](TlvWriter& nested) {
                if (plugin->contains("pluginId") && (*plugin)["pluginId"].is_string()) {
                    nested.writeString(2, (*plugin)["pluginId"].get<std::string>());
                }
                if (plugin->contains("format") && (*plugin)["format"].is_string()) {
                    nested.writeString(3, (*plugin)["format"].get<std::string>());
                }
                if (plugin->contains("displayName") && (*plugin)["displayName"].is_string()) {
                    nested.writeString(4, (*plugin)["displayName"].get<std::string>());
                }
                if (plugin->contains("manufacturer") && (*plugin)["manufacturer"].is_string()) {
                    nested.writeString(5, (*plugin)["manufacturer"].get<std::string>());
                }
            });
        }

        if (payload.contains("pluginId") && payload["pluginId"].is_string()) {
            writer.writeString(4, payload["pluginId"].get<std::string>());
        }

        return writer.intoBytes();
    } catch (const std::exception& ex) {
        error = ex.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodePluginRemovedOrUpdatedEvent(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter writer;
        writer.writeU32(1, 1);
        if (payload.contains("scanId") && payload["scanId"].is_string()) {
            writer.writeString(2, payload["scanId"].get<std::string>());
        }
        if (payload.contains("pluginId") && payload["pluginId"].is_string()) {
            writer.writeString(3, payload["pluginId"].get<std::string>());
        }
        return writer.intoBytes();
    } catch (const std::exception& ex) {
        error = ex.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodePluginScanCompletedEvent(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter writer;
        writer.writeU32(1, 1);

        if (payload.contains("scanId") && payload["scanId"].is_string()) {
            writer.writeString(2, payload["scanId"].get<std::string>());
        }

        if (payload.contains("summary") && payload["summary"].is_object()) {
            const auto& summary = payload["summary"];
            writer.writeObject(3, [&summary](TlvWriter& nested) {
                if (summary.contains("added") && summary["added"].is_number_unsigned()) {
                    nested.writeU64(2, summary["added"].get<std::uint64_t>());
                }
                if (summary.contains("removed") && summary["removed"].is_number_unsigned()) {
                    nested.writeU64(3, summary["removed"].get<std::uint64_t>());
                }
                if (summary.contains("updated") && summary["updated"].is_number_unsigned()) {
                    nested.writeU64(4, summary["updated"].get<std::uint64_t>());
                }
            });
        }

        return writer.intoBytes();
    } catch (const std::exception& ex) {
        error = ex.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodePluginScanFailedEvent(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter writer;
        writer.writeU32(1, 1);
        if (payload.contains("scanId") && payload["scanId"].is_string()) {
            writer.writeString(2, payload["scanId"].get<std::string>());
        }
        if (payload.contains("code") && payload["code"].is_string()) {
            writer.writeString(3, payload["code"].get<std::string>());
        }
        if (payload.contains("message") && payload["message"].is_string()) {
            writer.writeString(4, payload["message"].get<std::string>());
        }
        return writer.intoBytes();
    } catch (const std::exception& ex) {
        error = ex.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodePluginScanStatusEvent(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter writer;
        writer.writeU32(1, 1);

        if (payload.contains("scanId") && payload["scanId"].is_string()) {
            writer.writeString(2, payload["scanId"].get<std::string>());
        }
        if (payload.contains("state") && payload["state"].is_string()) {
            writer.writeString(3, payload["state"].get<std::string>());
        }
        if (payload.contains("pluginCount") && payload["pluginCount"].is_number_unsigned()) {
            writer.writeU64(4, payload["pluginCount"].get<std::uint64_t>());
        }
        if (payload.contains("clapCount") && payload["clapCount"].is_number_unsigned()) {
            writer.writeU64(5, payload["clapCount"].get<std::uint64_t>());
        }
        if (payload.contains("vst3Count") && payload["vst3Count"].is_number_unsigned()) {
            writer.writeU64(6, payload["vst3Count"].get<std::uint64_t>());
        }
        if (payload.contains("durationMs") && payload["durationMs"].is_number_unsigned()) {
            writer.writeU64(7, payload["durationMs"].get<std::uint64_t>());
        }
        if (payload.contains("message") && payload["message"].is_string()) {
            writer.writeString(8, payload["message"].get<std::string>());
        }
        if (payload.contains("scanLevel") && payload["scanLevel"].is_string()) {
            writer.writeString(9, payload["scanLevel"].get<std::string>());
        }

        return writer.intoBytes();
    } catch (const std::exception& ex) {
        error = ex.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodePluginCancelScanEvent(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter writer;
        writer.writeU32(1, 1);
        if (payload.contains("scanId") && payload["scanId"].is_string()) {
            writer.writeString(2, payload["scanId"].get<std::string>());
        }
        if (payload.contains("cancelled") && payload["cancelled"].is_boolean()) {
            writer.writeU32(3, payload["cancelled"].get<bool>() ? 1U : 0U);
        }
        return writer.intoBytes();
    } catch (const std::exception& ex) {
        error = ex.what();
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc::binary_envelope
