#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc::binary_envelope {
namespace {

std::optional<nlohmann::json> decodePlaybackSchedulePayloadTlv(
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
        nlohmann::json streams = nlohmann::json::array();
        nlohmann::json audioSegments = nlohmann::json::array();
        nlohmann::json midiEvents = nlohmann::json::array();
        nlohmann::json tempoMap = nlohmann::json::object();
        tempoMap["events"] = nlohmann::json::array();

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
                    throw std::runtime_error("streams list elementType must be TLV_OBJECT");
                }
                for (auto el : elements) {
                    nlohmann::json stream = nlohmann::json::object();
                    TlvReader sr(el);
                    while (true) {
                        auto shOpt = sr.readNextHeader();
                        if (!shOpt.has_value()) {
                            break;
                        }
                        TlvHeader sh = shOpt.value();
                        auto sv = sr.readValueBytes(sh.byteLen);
                        if (sh.fieldId == 2 && sh.fieldType == TLV_STRING) {
                            stream["streamId"] = readTlvString(sv);
                        } else if (sh.fieldId == 3 && sh.fieldType == TLV_STRING) {
                            stream["trackId"] = readTlvString(sv);
                        } else if (sh.fieldId == 4 && sh.fieldType == TLV_STRING) {
                            stream["laneId"] = readTlvString(sv);
                        } else if (sh.fieldId == 5 && sh.fieldType == TLV_STRING) {
                            stream["streamType"] = readTlvString(sv);
                        }
                    }
                    streams.push_back(stream);
                }
            } else if (h.fieldId == 5 && h.fieldType == TLV_LIST) {
                auto [elementType, elements] = readList(valueBytes);
                if (elementType != TLV_OBJECT) {
                    throw std::runtime_error("audioSegments list elementType must be TLV_OBJECT");
                }
                for (auto el : elements) {
                    nlohmann::json seg = nlohmann::json::object();
                    TlvReader ar(el);
                    while (true) {
                        auto ahOpt = ar.readNextHeader();
                        if (!ahOpt.has_value()) {
                            break;
                        }
                        TlvHeader ah = ahOpt.value();
                        auto av = ar.readValueBytes(ah.byteLen);
                        if (ah.fieldId == 2 && ah.fieldType == TLV_STRING) {
                            seg["streamId"] = readTlvString(av);
                        } else if (ah.fieldId == 3 && ah.fieldType == TLV_STRING) {
                            seg["assetId"] = readTlvString(av);
                        } else if (ah.fieldId == 4 && ah.fieldType == TLV_F64) {
                            seg["startBeats"] = readTlvF64(av);
                        } else if (ah.fieldId == 5 && ah.fieldType == TLV_F64) {
                            seg["endBeats"] = readTlvF64(av);
                        } else if (ah.fieldId == 6 && ah.fieldType == TLV_F64) {
                            seg["assetStartBeats"] = readTlvF64(av);
                        } else if (ah.fieldId == 7 && ah.fieldType == TLV_F64) {
                            seg["gainDb"] = readTlvF64(av);
                        } else if (ah.fieldId == 8 && ah.fieldType == TLV_F64) {
                            seg["fadeInBeats"] = readTlvF64(av);
                        } else if (ah.fieldId == 9 && ah.fieldType == TLV_F64) {
                            seg["fadeOutBeats"] = readTlvF64(av);
                        } else if (ah.fieldId == 10 && ah.fieldType == TLV_OBJECT) {
                            nlohmann::json stretch = nlohmann::json::object();
                            TlvReader str(av);
                            while (true) {
                                auto sthOpt = str.readNextHeader();
                                if (!sthOpt.has_value()) {
                                    break;
                                }
                                TlvHeader sth = sthOpt.value();
                                auto stv = str.readValueBytes(sth.byteLen);
                                if (sth.fieldId == 2 && sth.fieldType == TLV_STRING) {
                                    stretch["mode"] = readTlvString(stv);
                                } else if (sth.fieldId == 3 && sth.fieldType == TLV_F64) {
                                    stretch["ratio"] = readTlvF64(stv);
                                }
                            }
                            seg["stretch"] = stretch;
                        }
                    }
                    audioSegments.push_back(seg);
                }
            } else if (h.fieldId == 6 && h.fieldType == TLV_LIST) {
                auto [elementType, elements] = readList(valueBytes);
                if (elementType != TLV_OBJECT) {
                    throw std::runtime_error("midiEvents list elementType must be TLV_OBJECT");
                }
                for (auto el : elements) {
                    nlohmann::json ev = nlohmann::json::object();
                    TlvReader mr(el);
                    while (true) {
                        auto mhOpt = mr.readNextHeader();
                        if (!mhOpt.has_value()) {
                            break;
                        }
                        TlvHeader mh = mhOpt.value();
                        auto mv = mr.readValueBytes(mh.byteLen);
                        if (mh.fieldId == 2 && mh.fieldType == TLV_STRING) {
                            ev["streamId"] = readTlvString(mv);
                        } else if (mh.fieldId == 3 && mh.fieldType == TLV_F64) {
                            ev["timeBeats"] = readTlvF64(mv);
                        } else if (mh.fieldId == 4 && mh.fieldType == TLV_U32) {
                            ev["status"] = readTlvU32(mv);
                        } else if (mh.fieldId == 5 && mh.fieldType == TLV_U32) {
                            ev["data1"] = readTlvU32(mv);
                        } else if (mh.fieldId == 6 && mh.fieldType == TLV_U32) {
                            ev["data2"] = readTlvU32(mv);
                        } else if (mh.fieldId == 7 && mh.fieldType == TLV_U32) {
                            ev["channel"] = readTlvU32(mv);
                        }
                    }
                    midiEvents.push_back(ev);
                }
            } else if (h.fieldId == 7 && h.fieldType == TLV_F64) {
                out["startBeats"] = readTlvF64(valueBytes);
            } else if (h.fieldId == 8 && h.fieldType == TLV_F64) {
                out["endBeats"] = readTlvF64(valueBytes);
            }
        }

        out["tempoMap"] = tempoMap;
        out["streams"] = streams;
        out["audioSegments"] = audioSegments;
        out["midiEvents"] = midiEvents;
        return out;
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}
} // namespace

std::optional<nlohmann::json> decodePlaybackScheduleSnapshot(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    return decodePlaybackSchedulePayloadTlv(payloadBytes, error);
}

} // namespace loophole::signal::ipc::binary_envelope
