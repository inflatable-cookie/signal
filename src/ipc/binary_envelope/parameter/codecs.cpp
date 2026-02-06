#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<nlohmann::json> decodeParameterRequestDescriptors(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<nlohmann::json> decodeParameterRequestValues(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<nlohmann::json> decodeParameterSetValue(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
);

std::optional<std::vector<std::uint8_t>> encodeParameterDescriptorsSnapshot(
    const nlohmann::json& payload,
    std::string& error
);

std::optional<std::vector<std::uint8_t>> encodeParameterValuesSnapshot(
    const nlohmann::json& payload,
    std::string& error
);

std::optional<std::vector<std::uint8_t>> encodeParameterValueChanged(
    const nlohmann::json& payload,
    std::string& error
);

void appendParameterPayloadCodecs(std::vector<PayloadCodec>& out) {
    out.push_back(PayloadCodec{
        .domain = "parameter",
        .name = "requestDescriptors",
        .kind = IpcKind::Command,
        .decode = &decodeParameterRequestDescriptors,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "parameter",
        .name = "requestValues",
        .kind = IpcKind::Command,
        .decode = &decodeParameterRequestValues,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "parameter",
        .name = "setValue",
        .kind = IpcKind::Command,
        .decode = &decodeParameterSetValue,
        .encode = nullptr,
    });

    out.push_back(PayloadCodec{
        .domain = "parameter",
        .name = "descriptorsSnapshot",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeParameterDescriptorsSnapshot,
    });

    out.push_back(PayloadCodec{
        .domain = "parameter",
        .name = "valuesSnapshot",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeParameterValuesSnapshot,
    });

    out.push_back(PayloadCodec{
        .domain = "parameter",
        .name = "valueChanged",
        .kind = IpcKind::Event,
        .decode = nullptr,
        .encode = &encodeParameterValueChanged,
    });
}

} // namespace loophole::signal::ipc::binary_envelope
