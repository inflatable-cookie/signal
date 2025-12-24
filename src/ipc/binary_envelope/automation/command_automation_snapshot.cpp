#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc::binary_envelope {
namespace {

std::optional<nlohmann::json> decodeAutomationSnapshotPayloadTlv(
    std::span<const std::uint8_t> tlvBytes,
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

        nlohmann::json out = nlohmann::json::object();
        nlohmann::json tempoMap = nlohmann::json::object();
        tempoMap["events"] = nlohmann::json::array();
        nlohmann::json events = nlohmann::json::array();

        TlvReader r(tlvBytes);
        while (true) {
            auto hOpt = r.readNextHeader();
            if (!hOpt.has_value()) {
                break;
            }
            TlvHeader h = hOpt.value();
            auto valueBytes = r.readValueBytes(h.byteLen);

            if (h.fieldId == 2 && h.fieldType == TLV_STRING) {
                out["id"] = readTlvString(valueBytes);
            } else if (h.fieldId == 3 && h.fieldType == TLV_OBJECT) {
                TlvReader tr(valueBytes);
                while (true) {
                    auto thOpt = tr.readNextHeader();
                    if (!thOpt.has_value()) {
                        break;
                    }
                    TlvHeader th = thOpt.value();
                    auto tv = tr.readValueBytes(th.byteLen);
                    if (th.fieldId == 2 && th.fieldType == TLV_LIST) {
                        auto [elementType, elements] = readList(tv);
                        if (elementType != TLV_OBJECT) {
                            throw std::runtime_error("tempoMap.events list elementType must be TLV_OBJECT");
                        }
                        for (auto el : elements) {
                            nlohmann::json ev = nlohmann::json::object();
                            TlvReader er(el);
                            while (true) {
                                auto ehOpt = er.readNextHeader();
                                if (!ehOpt.has_value()) {
                                    break;
                                }
                                TlvHeader eh = ehOpt.value();
                                auto evv = er.readValueBytes(eh.byteLen);
                                if (eh.fieldId == 2 && eh.fieldType == TLV_F64) {
                                    ev["timeBeats"] = readTlvF64(evv);
                                } else if (eh.fieldId == 3 && eh.fieldType == TLV_F64) {
                                    ev["bpm"] = readTlvF64(evv);
                                } else if (eh.fieldId == 4 && eh.fieldType == TLV_U32) {
                                    ev["timeSigNumerator"] = readTlvU32(evv);
                                } else if (eh.fieldId == 5 && eh.fieldType == TLV_U32) {
                                    ev["timeSigDenominator"] = readTlvU32(evv);
                                }
                            }
                            tempoMap["events"].push_back(ev);
                        }
                    }
                }
            } else if (h.fieldId == 4 && h.fieldType == TLV_LIST) {
                auto [elementType, elements] = readList(valueBytes);
                if (elementType != TLV_OBJECT) {
                    throw std::runtime_error("automationSnapshot.events list elementType must be TLV_OBJECT");
                }
                for (auto el : elements) {
                    nlohmann::json ev = nlohmann::json::object();
                    TlvReader er(el);
                    while (true) {
                        auto ehOpt = er.readNextHeader();
                        if (!ehOpt.has_value()) {
                            break;
                        }
                        TlvHeader eh = ehOpt.value();
                        auto evv = er.readValueBytes(eh.byteLen);
                        if (eh.fieldId == 2 && eh.fieldType == TLV_F64) {
                            ev["timeBeats"] = readTlvF64(evv);
                        } else if (eh.fieldId == 3 && eh.fieldType == TLV_STRING) {
                            ev["nodeId"] = readTlvString(evv);
                        } else if (eh.fieldId == 4 && eh.fieldType == TLV_STRING) {
                            ev["paramId"] = readTlvString(evv);
                        } else if (eh.fieldId == 5 && eh.fieldType == TLV_F64) {
                            ev["valueNorm"] = readTlvF64(evv);
                        } else if (eh.fieldId == 6 && eh.fieldType == TLV_STRING) {
                            ev["curve"] = readTlvString(evv);
                        }
                    }
                    events.push_back(ev);
                }
            }
        }

        out["tempoMap"] = tempoMap;
        out["events"] = events;
        return out;
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace

std::optional<nlohmann::json> decodeAutomationSnapshot(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    return decodeAutomationSnapshotPayloadTlv(payloadBytes, error);
}

} // namespace loophole::signal::ipc::binary_envelope
