#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecsInternal.hpp"
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc::binary_envelope {
namespace {

std::optional<nlohmann::json> decodeGraphSnapshotPayloadTlv(
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
        nlohmann::json nodes = nlohmann::json::array();
        nlohmann::json connections = nlohmann::json::array();

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
            } else if (h.fieldId == 3 && h.fieldType == TLV_LIST) {
                auto [elementType, elements] = readList(valueBytes);
                if (elementType != TLV_OBJECT) {
                    throw std::runtime_error("graphSnapshot.nodes list elementType must be TLV_OBJECT");
                }
                for (auto el : elements) {
                    nlohmann::json node = nlohmann::json::object();
                    TlvReader nr(el);
                    while (true) {
                        auto nhOpt = nr.readNextHeader();
                        if (!nhOpt.has_value()) {
                            break;
                        }
                        TlvHeader nh = nhOpt.value();
                        auto nv = nr.readValueBytes(nh.byteLen);
                        if (nh.fieldId == 2 && nh.fieldType == TLV_STRING) {
                            node["nodeId"] = readTlvString(nv);
                        } else if (nh.fieldId == 3 && nh.fieldType == TLV_STRING) {
                            node["kind"] = readTlvString(nv);
                        } else if (nh.fieldId == 4 && nh.fieldType == TLV_STRING) {
                            node["trackId"] = readTlvString(nv);
                        } else if (nh.fieldId == 5 && nh.fieldType == TLV_STRING) {
                            node["laneId"] = readTlvString(nv);
                        } else if (nh.fieldId == 18 && nh.fieldType == TLV_BOOL) {
                            node["laneHasContent"] = readTlvBool(nv);
                        } else if (nh.fieldId == 19 && nh.fieldType == TLV_BOOL) {
                            node["lanePersistent"] = readTlvBool(nv);
                        } else if (nh.fieldId == 6 && nh.fieldType == TLV_STRING) {
                            node["pluginFormat"] = readTlvString(nv);
                        } else if (nh.fieldId == 7 && nh.fieldType == TLV_STRING) {
                            node["pluginId"] = readTlvString(nv);
                        } else if (nh.fieldId == 8 && nh.fieldType == TLV_OBJECT) {
                            nlohmann::json audio = nlohmann::json::object();
                            TlvReader ar(nv);
                            while (true) {
                                auto ahOpt = ar.readNextHeader();
                                if (!ahOpt.has_value()) {
                                    break;
                                }
                                TlvHeader ah = ahOpt.value();
                                auto av = ar.readValueBytes(ah.byteLen);
                                if (ah.fieldId == 2 && ah.fieldType == TLV_U32) {
                                    audio["inputs"] = readTlvU32(av);
                                } else if (ah.fieldId == 3 && ah.fieldType == TLV_U32) {
                                    audio["outputs"] = readTlvU32(av);
                                } else if (ah.fieldId == 4 && ah.fieldType == TLV_STRING) {
                                    audio["layout"] = readTlvString(av);
                                }
                            }
                            node["audio"] = audio;
                        } else if (nh.fieldId == 9 && nh.fieldType == TLV_U32) {
                            node["numAudioInputs"] = readTlvU32(nv);
                        } else if (nh.fieldId == 10 && nh.fieldType == TLV_U32) {
                            node["numAudioOutputs"] = readTlvU32(nv);
                        } else if (nh.fieldId == 11 && nh.fieldType == TLV_U32) {
                            node["numMidiInputs"] = readTlvU32(nv);
                        } else if (nh.fieldId == 12 && nh.fieldType == TLV_U32) {
                            node["numMidiOutputs"] = readTlvU32(nv);
                        } else if (nh.fieldId == 13 && nh.fieldType == TLV_U32) {
                            node["latencySamples"] = readTlvU32(nv);
                        } else if (nh.fieldId == 14 && nh.fieldType == TLV_U32) {
                            node["tailSamples"] = readTlvU32(nv);
                        } else if (nh.fieldId == 15 && nh.fieldType == TLV_OBJECT) {
                            nlohmann::json mix = nlohmann::json::object();
                            TlvReader mr(nv);
                            while (true) {
                                auto mhOpt = mr.readNextHeader();
                                if (!mhOpt.has_value()) {
                                    break;
                                }
                                TlvHeader mh = mhOpt.value();
                                auto mv = mr.readValueBytes(mh.byteLen);
                                if (mh.fieldId == 2 && mh.fieldType == TLV_F64) {
                                    mix["gain"] = readTlvF64(mv);
                                }
                            }
                            node["mix"] = mix;
                        } else if (nh.fieldId == 21 && nh.fieldType == TLV_OBJECT) {
                            nlohmann::json spatial = nlohmann::json::object();
                            TlvReader sr(nv);
                            while (true) {
                                auto shOpt = sr.readNextHeader();
                                if (!shOpt.has_value()) {
                                    break;
                                }
                                TlvHeader sh = shOpt.value();
                                auto sv = sr.readValueBytes(sh.byteLen);
                                if (sh.fieldId == 2 && sh.fieldType == TLV_BOOL) {
                                    spatial["enabled"] = readTlvBool(sv);
                                } else if (sh.fieldId == 3 && sh.fieldType == TLV_STRING) {
                                    spatial["adapter"] = readTlvString(sv);
                                } else if (sh.fieldId == 4 && sh.fieldType == TLV_OBJECT) {
                                    nlohmann::json options = nlohmann::json::object();
                                    TlvReader orr(sv);
                                    while (true) {
                                        auto ohOpt = orr.readNextHeader();
                                        if (!ohOpt.has_value()) {
                                            break;
                                        }
                                        TlvHeader oh = ohOpt.value();
                                        auto ov = orr.readValueBytes(oh.byteLen);
                                        if (oh.fieldId == 2 && oh.fieldType == TLV_STRING) {
                                            options["mixPolicy"] = readTlvString(ov);
                                        }
                                    }
                                    spatial["options"] = options;
                                }
                            }
                            node["spatial"] = spatial;
                        } else if (nh.fieldId == 16 && nh.fieldType == TLV_OBJECT) {
                            nlohmann::json channel = nlohmann::json::object();
                            TlvReader cr(nv);
                            while (true) {
                                auto chOpt = cr.readNextHeader();
                                if (!chOpt.has_value()) {
                                    break;
                                }
                                TlvHeader ch = chOpt.value();
                                auto cv = cr.readValueBytes(ch.byteLen);
                                if (ch.fieldId == 2 && ch.fieldType == TLV_STRING) {
                                    channel["channelId"] = readTlvString(cv);
                                } else if (ch.fieldId == 3 && ch.fieldType == TLV_STRING) {
                                    channel["role"] = readTlvString(cv);
                                } else if (ch.fieldId == 4 && ch.fieldType == TLV_BOOL) {
                                    channel["trackOwned"] = readTlvBool(cv);
                                } else if (ch.fieldId == 5 && ch.fieldType == TLV_BOOL) {
                                    channel["canDelete"] = readTlvBool(cv);
                                }
                            }
                            node["channel"] = channel;
                        } else if (nh.fieldId == 20 && nh.fieldType == TLV_OBJECT) {
                            nlohmann::json send = nlohmann::json::object();
                            TlvReader sr(nv);
                            while (true) {
                                auto shOpt = sr.readNextHeader();
                                if (!shOpt.has_value()) {
                                    break;
                                }
                                TlvHeader sh = shOpt.value();
                                auto sv = sr.readValueBytes(sh.byteLen);
                                if (sh.fieldId == 2 && sh.fieldType == TLV_STRING) {
                                    send["sourceChannelId"] = readTlvString(sv);
                                } else if (sh.fieldId == 3 && sh.fieldType == TLV_STRING) {
                                    send["targetChannelId"] = readTlvString(sv);
                                } else if (sh.fieldId == 4 && sh.fieldType == TLV_BOOL) {
                                    send["preFader"] = readTlvBool(sv);
                                }
                            }
                            node["send"] = send;
                        } else if (nh.fieldId == 17 && nh.fieldType == TLV_OBJECT) {
                            nlohmann::json device = nlohmann::json::object();
                            TlvReader dr(nv);
                            while (true) {
                                auto dhOpt = dr.readNextHeader();
                                if (!dhOpt.has_value()) {
                                    break;
                                }
                                TlvHeader dh = dhOpt.value();
                                auto dv = dr.readValueBytes(dh.byteLen);
                                if (dh.fieldId == 2 && dh.fieldType == TLV_STRING) {
                                    device["deviceId"] = readTlvString(dv);
                                } else if (dh.fieldId == 3 && dh.fieldType == TLV_BOOL) {
                                    device["isDefault"] = readTlvBool(dv);
                                }
                            }
                            node["device"] = device;
                        }
                    }
                    nodes.push_back(node);
                }
            } else if (h.fieldId == 4 && h.fieldType == TLV_LIST) {
                auto [elementType, elements] = readList(valueBytes);
                if (elementType != TLV_OBJECT) {
                    throw std::runtime_error("graphSnapshot.connections list elementType must be TLV_OBJECT");
                }
                for (auto el : elements) {
                    nlohmann::json conn = nlohmann::json::object();
                    TlvReader cr(el);
                    while (true) {
                        auto chOpt = cr.readNextHeader();
                        if (!chOpt.has_value()) {
                            break;
                        }
                        TlvHeader ch = chOpt.value();
                        auto cv = cr.readValueBytes(ch.byteLen);
                        if (ch.fieldId == 2 && ch.fieldType == TLV_STRING) {
                            conn["fromStreamId"] = readTlvString(cv);
                        } else if (ch.fieldId == 3 && ch.fieldType == TLV_STRING) {
                            conn["fromNodeId"] = readTlvString(cv);
                        } else if (ch.fieldId == 4 && ch.fieldType == TLV_U32) {
                            conn["fromOutputIndex"] = readTlvU32(cv);
                        } else if (ch.fieldId == 5 && ch.fieldType == TLV_STRING) {
                            conn["toNodeId"] = readTlvString(cv);
                        } else if (ch.fieldId == 6 && ch.fieldType == TLV_U32) {
                            conn["toInputIndex"] = readTlvU32(cv);
                        }
                    }
                    connections.push_back(conn);
                }
            }
        }

        out["nodes"] = nodes;
        out["connections"] = connections;
        return out;
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}
} // namespace

std::optional<nlohmann::json> decodeGraphSnapshot(
    std::span<const std::uint8_t> payloadBytes,
    std::string& error
) {
    return decodeGraphSnapshotPayloadTlv(payloadBytes, error);
}

} // namespace loophole::signal::ipc::binary_envelope
