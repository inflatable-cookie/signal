#include "ipc/binary_envelope/CodecCommon.hpp"

namespace loophole::signal::ipc::binary_envelope {
namespace {

std::optional<nlohmann::json> decodeSchemaOnly(
    std::span<const std::uint8_t>,
    std::string&
) {
    return nlohmann::json::object();
}

std::optional<nlohmann::json> decodeRescanCommand(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    try {
        TlvReader reader(payloadBytes);
        std::vector<std::string> formats;
        bool force = false;
        std::optional<std::string> scanId = std::nullopt;
        std::optional<std::string> scanLevel = std::nullopt;
        nlohmann::json lightCache = nlohmann::json::array();

        while (auto header = reader.readNextHeader()) {
            auto valueBytes = reader.readValueBytes(header->byteLen);
            if (header->fieldId == 2 && header->fieldType == TLV_LIST) {
                BinaryReader listReader(valueBytes);
                auto elementType = listReader.readU8("formats.elementType");
                auto count = listReader.readU32Le("formats.count");
                if (elementType != TLV_STRING) {
                    continue;
                }

                for (std::uint32_t idx = 0; idx < count; ++idx) {
                    auto len = listReader.readU32Le("formats.length");
                    auto entryBytes = listReader.readSlice(len, "formats.entry");
                    BinaryReader strReader(entryBytes);
                    formats.push_back(strReader.readStringU16Len("formats.value"));
                }
            } else if (header->fieldId == 3 && header->fieldType == TLV_U32) {
                BinaryReader valueReader(valueBytes);
                force = valueReader.readU32Le("force") != 0;
            } else if (header->fieldId == 4 && header->fieldType == TLV_STRING) {
                BinaryReader valueReader(valueBytes);
                scanId = valueReader.readStringU16Len("scanId");
            } else if (header->fieldId == 5 && header->fieldType == TLV_STRING) {
                BinaryReader valueReader(valueBytes);
                scanLevel = valueReader.readStringU16Len("scanLevel");
            } else if (header->fieldId == 6 && header->fieldType == TLV_LIST) {
                BinaryReader listReader(valueBytes);
                auto elementType = listReader.readU8("lightCache.elementType");
                auto count = listReader.readU32Le("lightCache.count");
                if (elementType != TLV_OBJECT) {
                    continue;
                }
                for (std::uint32_t idx = 0; idx < count; ++idx) {
                    auto len = listReader.readU32Le("lightCache.length");
                    auto entryBytes = listReader.readSlice(len, "lightCache.entry");
                    TlvReader entryReader(entryBytes);
                    nlohmann::json entry = nlohmann::json::object();
                    while (auto entryHeader = entryReader.readNextHeader()) {
                        auto entryValueBytes = entryReader.readValueBytes(entryHeader->byteLen);
                        if (entryHeader->fieldId == 2 && entryHeader->fieldType == TLV_STRING) {
                            BinaryReader valueReader(entryValueBytes);
                            entry["pluginId"] = valueReader.readStringU16Len("lightCache.pluginId");
                        } else if (entryHeader->fieldId == 3 && entryHeader->fieldType == TLV_STRING) {
                            BinaryReader valueReader(entryValueBytes);
                            entry["binaryPath"] = valueReader.readStringU16Len("lightCache.binaryPath");
                        } else if (entryHeader->fieldId == 4 && entryHeader->fieldType == TLV_U64) {
                            BinaryReader valueReader(entryValueBytes);
                            entry["fileMtimeUnix"] = valueReader.readU64Le("lightCache.fileMtimeUnix");
                        } else if (entryHeader->fieldId == 5 && entryHeader->fieldType == TLV_U64) {
                            BinaryReader valueReader(entryValueBytes);
                            entry["fileSizeBytes"] = valueReader.readU64Le("lightCache.fileSizeBytes");
                        } else if (entryHeader->fieldId == 6 && entryHeader->fieldType == TLV_STRING) {
                            BinaryReader valueReader(entryValueBytes);
                            entry["fileSignatureHash"] = valueReader.readStringU16Len("lightCache.fileSignatureHash");
                        }
                    }
                    lightCache.push_back(std::move(entry));
                }
            }
        }

        nlohmann::json out = nlohmann::json::object();
        if (scanId.has_value()) {
            out["scanId"] = scanId.value();
        }
        if (scanLevel.has_value()) {
            out["scanLevel"] = scanLevel.value();
        }
        out["options"] = nlohmann::json{
            {"formats", formats},
            {"force", force},
            {"lightCache", lightCache},
        };
        return out;
    } catch (const std::exception& ex) {
        error = ex.what();
        return std::nullopt;
    }
}

} // namespace

std::optional<nlohmann::json> decodePluginListCommand(
    std::span<const std::uint8_t>,
    std::string&
) {
    return nlohmann::json::object();
}

std::optional<nlohmann::json> decodePluginRescanCommand(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    return decodeRescanCommand(payloadBytes, error);
}

std::optional<nlohmann::json> decodePluginCancelScanCommand(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    return decodeSchemaOnly(payloadBytes, error);
}

std::optional<nlohmann::json> decodePluginScanStatusCommand(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    return decodeSchemaOnly(payloadBytes, error);
}

} // namespace loophole::signal::ipc::binary_envelope
