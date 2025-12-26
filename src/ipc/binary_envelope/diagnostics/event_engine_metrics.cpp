#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<std::vector<std::uint8_t>> encodeDiagnosticsEngineMetrics(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter w;
        w.writeU32(1, 1);

        if (payload.contains("cpuLoad") && payload["cpuLoad"].is_number()) {
            w.writeF64(2, payload["cpuLoad"].get<double>());
        }
        if (payload.contains("xruns") && payload["xruns"].is_number()) {
            w.writeU32(3, static_cast<std::uint32_t>(payload["xruns"].get<std::uint64_t>()));
        }
        if (payload.contains("engineState") && payload["engineState"].is_string()) {
            w.writeString(4, payload["engineState"].get<std::string>());
        }
        if (payload.contains("transportState") && payload["transportState"].is_string()) {
            w.writeString(5, payload["transportState"].get<std::string>());
        }
        if (payload.contains("sampleRate") && payload["sampleRate"].is_number()) {
            w.writeF64(6, payload["sampleRate"].get<double>());
        }
        if (payload.contains("blockSize") && payload["blockSize"].is_number()) {
            w.writeU32(7, static_cast<std::uint32_t>(payload["blockSize"].get<std::uint64_t>()));
        }

        // Plugin scanning (Signal-only background work).
        if (payload.contains("pluginScanState") && payload["pluginScanState"].is_string()) {
            w.writeString(10, payload["pluginScanState"].get<std::string>());
        }
        if (payload.contains("pluginScanPluginCount") && payload["pluginScanPluginCount"].is_number()) {
            w.writeU32(
                11,
                static_cast<std::uint32_t>(payload["pluginScanPluginCount"].get<std::uint64_t>())
            );
        }
        if (payload.contains("pluginScanLastError") && payload["pluginScanLastError"].is_string()) {
            w.writeString(12, payload["pluginScanLastError"].get<std::string>());
        }
        if (payload.contains("pluginScanDurationMs") && payload["pluginScanDurationMs"].is_number()) {
            w.writeU32(
                13,
                static_cast<std::uint32_t>(payload["pluginScanDurationMs"].get<std::uint64_t>())
            );
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc::binary_envelope
