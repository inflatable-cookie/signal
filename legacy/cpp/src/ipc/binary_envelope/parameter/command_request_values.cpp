#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"

namespace loophole::signal::ipc::binary_envelope {

std::optional<nlohmann::json> decodeParameterRequestValues(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    try {
        auto readList = [](std::span<const std::uint8_t> bytes) -> std::pair<std::uint8_t, std::vector<std::span<const std::uint8_t>>> {
            if (bytes.size() < 5) {
                throw std::runtime_error("Invalid TLV list: truncated");
            }

            std::uint8_t elementType = bytes[0];
            std::uint32_t count = 0;
            count |= static_cast<std::uint32_t>(bytes[1]);
            count |= static_cast<std::uint32_t>(bytes[2]) << 8;
            count |= static_cast<std::uint32_t>(bytes[3]) << 16;
            count |= static_cast<std::uint32_t>(bytes[4]) << 24;

            std::size_t offset = 5;
            std::vector<std::span<const std::uint8_t>> elements;
            elements.reserve(count);
            for (std::uint32_t i = 0; i < count; i++) {
                if (bytes.size() - offset < 4) {
                    throw std::runtime_error("Invalid TLV list: element length truncated");
                }

                std::uint32_t len = 0;
                len |= static_cast<std::uint32_t>(bytes[offset]);
                len |= static_cast<std::uint32_t>(bytes[offset + 1]) << 8;
                len |= static_cast<std::uint32_t>(bytes[offset + 2]) << 16;
                len |= static_cast<std::uint32_t>(bytes[offset + 3]) << 24;
                offset += 4;

                if (bytes.size() - offset < len) {
                    throw std::runtime_error("Invalid TLV list: element bytes truncated");
                }

                elements.push_back(bytes.subspan(offset, len));
                offset += len;
            }

            if (offset != bytes.size()) {
                throw std::runtime_error("Invalid TLV list: trailing bytes");
            }

            return {elementType, elements};
        };

        TlvReader r(payloadBytes);
        nlohmann::json payload = nlohmann::json::object();

        while (true) {
            auto header = r.readNextHeader();
            if (!header.has_value()) {
                break;
            }

            auto h = header.value();
            auto valueBytes = r.readValueBytes(h.byteLen);

            if (h.fieldId == 2 && h.fieldType == TLV_OBJECT) {
                TlvReader scopeReader(valueBytes);
                nlohmann::json scope = nlohmann::json::object();

                while (true) {
                    auto scopeHeader = scopeReader.readNextHeader();
                    if (!scopeHeader.has_value()) {
                        break;
                    }

                    auto sh = scopeHeader.value();
                    auto scopeValueBytes = scopeReader.readValueBytes(sh.byteLen);

                    if (sh.fieldId == 2 && sh.fieldType == TLV_STRING) {
                        scope["nodeId"] = readTlvString(scopeValueBytes);
                    } else if (sh.fieldId == 3 && sh.fieldType == TLV_STRING) {
                        scope["pluginInstanceId"] = readTlvString(scopeValueBytes);
                    }
                }

                payload["scope"] = scope;
            } else if (h.fieldId == 3 && h.fieldType == TLV_LIST) {
                auto [elementType, elements] = readList(valueBytes);
                if (elementType == TLV_STRING) {
                    nlohmann::json paramIds = nlohmann::json::array();
                    for (const auto& entry : elements) {
                        paramIds.push_back(readTlvString(entry));
                    }
                    payload["paramIds"] = paramIds;
                }
            }
        }

        return payload;
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc::binary_envelope
