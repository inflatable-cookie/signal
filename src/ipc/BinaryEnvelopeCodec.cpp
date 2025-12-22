#include "ipc/BinaryEnvelopeCodec.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <cstring>
#include <stdexcept>
#include <limits>

namespace loophole::signal::ipc {
namespace {

class BinaryReader {
public:
    explicit BinaryReader(std::span<const std::uint8_t> bytes)
        : bytes_(bytes) {}

    std::size_t remaining() const {
        return bytes_.size() - offset_;
    }

    std::uint8_t readU8(std::string_view what) {
        if (remaining() < 1) {
            throw std::runtime_error(std::string("Unexpected EOF (") + std::string(what) + ")");
        }

        std::uint8_t v = bytes_[offset_];
        offset_ += 1;
        return v;
    }

    std::uint16_t readU16Le(std::string_view what) {
        if (remaining() < 2) {
            throw std::runtime_error(std::string("Unexpected EOF (") + std::string(what) + ")");
        }

        std::uint16_t v = 0;
        v |= static_cast<std::uint16_t>(bytes_[offset_]);
        v |= static_cast<std::uint16_t>(bytes_[offset_ + 1]) << 8;
        offset_ += 2;
        return v;
    }

    std::uint32_t readU32Le(std::string_view what) {
        if (remaining() < 4) {
            throw std::runtime_error(std::string("Unexpected EOF (") + std::string(what) + ")");
        }

        std::uint32_t v = 0;
        v |= static_cast<std::uint32_t>(bytes_[offset_]);
        v |= static_cast<std::uint32_t>(bytes_[offset_ + 1]) << 8;
        v |= static_cast<std::uint32_t>(bytes_[offset_ + 2]) << 16;
        v |= static_cast<std::uint32_t>(bytes_[offset_ + 3]) << 24;
        offset_ += 4;
        return v;
    }

    std::uint64_t readU64Le(std::string_view what) {
        if (remaining() < 8) {
            throw std::runtime_error(std::string("Unexpected EOF (") + std::string(what) + ")");
        }

        std::uint64_t v = 0;
        for (int i = 0; i < 8; i++) {
            v |= static_cast<std::uint64_t>(bytes_[offset_ + static_cast<std::size_t>(i)]) << (8 * i);
        }
        offset_ += 8;
        return v;
    }

    std::string readStringU16Len(std::string_view what) {
        std::uint16_t len = readU16Le(what);
        if (remaining() < len) {
            throw std::runtime_error(std::string("Unexpected EOF (") + std::string(what) + ")");
        }

        std::string out(reinterpret_cast<const char*>(&bytes_[offset_]), len);
        offset_ += len;
        return out;
    }

    std::optional<std::string> readOptionalStringU16Len(std::string_view what) {
        std::uint8_t tag = readU8(what);
        if (tag == 0) {
            return std::nullopt;
        }

        return readStringU16Len(what);
    }

    std::span<const std::uint8_t> readSlice(std::size_t len, std::string_view what) {
        if (remaining() < len) {
            throw std::runtime_error(std::string("Unexpected EOF (") + std::string(what) + ")");
        }

        auto out = bytes_.subspan(offset_, len);
        offset_ += len;
        return out;
    }

private:
    std::span<const std::uint8_t> bytes_;
    std::size_t offset_ = 0;
};

class BinaryWriter {
public:
    void writeU8(std::uint8_t v) {
        out_.push_back(v);
    }

    void writeU16Le(std::uint16_t v) {
        out_.push_back(static_cast<std::uint8_t>(v & 0xff));
        out_.push_back(static_cast<std::uint8_t>((v >> 8) & 0xff));
    }

    void writeU32Le(std::uint32_t v) {
        out_.push_back(static_cast<std::uint8_t>(v & 0xff));
        out_.push_back(static_cast<std::uint8_t>((v >> 8) & 0xff));
        out_.push_back(static_cast<std::uint8_t>((v >> 16) & 0xff));
        out_.push_back(static_cast<std::uint8_t>((v >> 24) & 0xff));
    }

    void writeBytes(std::span<const std::uint8_t> bytes) {
        out_.insert(out_.end(), bytes.begin(), bytes.end());
    }

    void writeStringU16Len(const std::string& s) {
        if (s.size() > 0xffff) {
            throw std::runtime_error("String too large for u16 length");
        }

        writeU16Le(static_cast<std::uint16_t>(s.size()));
        writeBytes(std::span<const std::uint8_t>(reinterpret_cast<const std::uint8_t*>(s.data()), s.size()));
    }

    void writeOptionalStringU16Len(const std::optional<std::string>& s) {
        if (!s.has_value()) {
            writeU8(0);
            return;
        }

        writeU8(1);
        writeStringU16Len(s.value());
    }

    std::vector<std::uint8_t> intoBytes() {
        return std::move(out_);
    }

private:
    std::vector<std::uint8_t> out_;
};

struct TlvHeader {
    std::uint16_t fieldId;
    std::uint8_t fieldType;
    std::uint32_t byteLen;
};

class TlvReader {
public:
    explicit TlvReader(std::span<const std::uint8_t> bytes)
        : bytes_(bytes) {}

    std::optional<TlvHeader> readNextHeader() {
        if (offset_ == bytes_.size()) {
            return std::nullopt;
        }

        if (bytes_.size() - offset_ < 7) {
            throw std::runtime_error("Invalid TLV header: truncated");
        }

        std::uint16_t fieldId = 0;
        fieldId |= static_cast<std::uint16_t>(bytes_[offset_]);
        fieldId |= static_cast<std::uint16_t>(bytes_[offset_ + 1]) << 8;
        std::uint8_t fieldType = bytes_[offset_ + 2];
        std::uint32_t byteLen = 0;
        byteLen |= static_cast<std::uint32_t>(bytes_[offset_ + 3]);
        byteLen |= static_cast<std::uint32_t>(bytes_[offset_ + 4]) << 8;
        byteLen |= static_cast<std::uint32_t>(bytes_[offset_ + 5]) << 16;
        byteLen |= static_cast<std::uint32_t>(bytes_[offset_ + 6]) << 24;
        offset_ += 7;

        if (bytes_.size() - offset_ < byteLen) {
            throw std::runtime_error("Invalid TLV value: truncated");
        }

        return TlvHeader{fieldId, fieldType, byteLen};
    }

    std::span<const std::uint8_t> readValueBytes(std::uint32_t byteLen) {
        auto out = bytes_.subspan(offset_, byteLen);
        offset_ += byteLen;
        return out;
    }

private:
    std::span<const std::uint8_t> bytes_;
    std::size_t offset_ = 0;
};

// Must match `echo-ipc-tlv` (see `echo/crates/echo-ipc-tlv/src/tlv.rs`).
constexpr std::uint8_t TLV_BOOL = 0x01;
constexpr std::uint8_t TLV_U32 = 0x02;
constexpr std::uint8_t TLV_I32 = 0x03;
constexpr std::uint8_t TLV_U64 = 0x04;
constexpr std::uint8_t TLV_F64 = 0x06;
constexpr std::uint8_t TLV_STRING = 0x07;
constexpr std::uint8_t TLV_BYTES = 0x08;
constexpr std::uint8_t TLV_OBJECT = 0x09;
constexpr std::uint8_t TLV_LIST = 0x0a;

class TlvWriter {
public:
    void writeHeader(std::uint16_t fieldId, std::uint8_t fieldType, std::uint32_t byteLen) {
        out_.push_back(static_cast<std::uint8_t>(fieldId & 0xff));
        out_.push_back(static_cast<std::uint8_t>((fieldId >> 8) & 0xff));
        out_.push_back(fieldType);
        out_.push_back(static_cast<std::uint8_t>(byteLen & 0xff));
        out_.push_back(static_cast<std::uint8_t>((byteLen >> 8) & 0xff));
        out_.push_back(static_cast<std::uint8_t>((byteLen >> 16) & 0xff));
        out_.push_back(static_cast<std::uint8_t>((byteLen >> 24) & 0xff));
    }

    void writeBool(std::uint16_t fieldId, bool v) {
        writeHeader(fieldId, TLV_BOOL, 1);
        out_.push_back(v ? 1 : 0);
    }

    void writeU32(std::uint16_t fieldId, std::uint32_t v) {
        writeHeader(fieldId, TLV_U32, 4);
        out_.push_back(static_cast<std::uint8_t>(v & 0xff));
        out_.push_back(static_cast<std::uint8_t>((v >> 8) & 0xff));
        out_.push_back(static_cast<std::uint8_t>((v >> 16) & 0xff));
        out_.push_back(static_cast<std::uint8_t>((v >> 24) & 0xff));
    }

    void writeU64(std::uint16_t fieldId, std::uint64_t v) {
        writeHeader(fieldId, TLV_U64, 8);
        for (int i = 0; i < 8; i++) {
            out_.push_back(static_cast<std::uint8_t>((v >> (8 * i)) & 0xff));
        }
    }

    void writeF64(std::uint16_t fieldId, double v) {
        writeHeader(fieldId, TLV_F64, 8);
        std::uint64_t bits = 0;
        static_assert(sizeof(double) == sizeof(std::uint64_t));
        std::memcpy(&bits, &v, sizeof(double));
        for (int i = 0; i < 8; i++) {
            out_.push_back(static_cast<std::uint8_t>((bits >> (8 * i)) & 0xff));
        }
    }

    void writeString(std::uint16_t fieldId, const std::string& value) {
        if (value.size() > 0xffff) {
            throw std::runtime_error("String too large for u16 length");
        }

        std::vector<std::uint8_t> inner;
        inner.reserve(2 + value.size());
        inner.push_back(static_cast<std::uint8_t>(value.size() & 0xff));
        inner.push_back(static_cast<std::uint8_t>((value.size() >> 8) & 0xff));
        inner.insert(
            inner.end(),
            reinterpret_cast<const std::uint8_t*>(value.data()),
            reinterpret_cast<const std::uint8_t*>(value.data()) + value.size()
        );

        writeHeader(fieldId, TLV_STRING, static_cast<std::uint32_t>(inner.size()));
        out_.insert(out_.end(), inner.begin(), inner.end());
    }

    template <typename Fn>
    void writeObject(std::uint16_t fieldId, Fn writeInner) {
        TlvWriter inner;
        writeInner(inner);
        auto innerBytes = inner.intoBytes();
        writeHeader(fieldId, TLV_OBJECT, static_cast<std::uint32_t>(innerBytes.size()));
        out_.insert(out_.end(), innerBytes.begin(), innerBytes.end());
    }

    void writeList(std::uint16_t fieldId, std::uint8_t elementType, const std::vector<std::vector<std::uint8_t>>& elements) {
        std::uint64_t byteLen = 1 + 4;
        for (const auto& el : elements) {
            byteLen += 4;
            byteLen += el.size();
        }

        if (byteLen > 0xffff'ffffULL) {
            throw std::runtime_error("TLV list too large");
        }

        writeHeader(fieldId, TLV_LIST, static_cast<std::uint32_t>(byteLen));
        out_.push_back(elementType);

        std::uint32_t count = static_cast<std::uint32_t>(elements.size());
        out_.push_back(static_cast<std::uint8_t>(count & 0xff));
        out_.push_back(static_cast<std::uint8_t>((count >> 8) & 0xff));
        out_.push_back(static_cast<std::uint8_t>((count >> 16) & 0xff));
        out_.push_back(static_cast<std::uint8_t>((count >> 24) & 0xff));

        for (const auto& el : elements) {
            std::uint32_t len = static_cast<std::uint32_t>(el.size());
            out_.push_back(static_cast<std::uint8_t>(len & 0xff));
            out_.push_back(static_cast<std::uint8_t>((len >> 8) & 0xff));
            out_.push_back(static_cast<std::uint8_t>((len >> 16) & 0xff));
            out_.push_back(static_cast<std::uint8_t>((len >> 24) & 0xff));
            out_.insert(out_.end(), el.begin(), el.end());
        }
    }

    void writeObjectList(std::uint16_t fieldId, const std::vector<std::vector<std::uint8_t>>& elements) {
        writeList(fieldId, TLV_OBJECT, elements);
    }

    std::vector<std::uint8_t> intoBytes() {
        return std::move(out_);
    }

private:
    std::vector<std::uint8_t> out_;
};

std::string readTlvString(std::span<const std::uint8_t> bytes) {
    if (bytes.size() < 2) {
        throw std::runtime_error("Invalid TLV string: missing u16 length");
    }

    std::uint16_t len = 0;
    len |= static_cast<std::uint16_t>(bytes[0]);
    len |= static_cast<std::uint16_t>(bytes[1]) << 8;

    if (bytes.size() - 2 < len) {
        throw std::runtime_error("Invalid TLV string: truncated");
    }

    return std::string(reinterpret_cast<const char*>(bytes.data() + 2), len);
}

std::uint32_t readTlvU32(std::span<const std::uint8_t> bytes) {
    if (bytes.size() != 4) {
        throw std::runtime_error("Invalid TLV u32: wrong length");
    }

    std::uint32_t v = 0;
    v |= static_cast<std::uint32_t>(bytes[0]);
    v |= static_cast<std::uint32_t>(bytes[1]) << 8;
    v |= static_cast<std::uint32_t>(bytes[2]) << 16;
    v |= static_cast<std::uint32_t>(bytes[3]) << 24;
    return v;
}

bool readTlvBool(std::span<const std::uint8_t> bytes) {
    if (bytes.size() != 1) {
        throw std::runtime_error("Invalid TLV bool: wrong length");
    }

    return bytes[0] != 0;
}

std::uint64_t readTlvU64(std::span<const std::uint8_t> bytes) {
    if (bytes.size() != 8) {
        throw std::runtime_error("Invalid TLV u64: wrong length");
    }

    std::uint64_t v = 0;
    for (int i = 0; i < 8; i++) {
        v |= static_cast<std::uint64_t>(bytes[static_cast<std::size_t>(i)]) << (8 * i);
    }
    return v;
}

double readTlvF64(std::span<const std::uint8_t> bytes) {
    if (bytes.size() != 8) {
        throw std::runtime_error("Invalid TLV f64: wrong length");
    }

    std::uint64_t bits = 0;
    for (int i = 0; i < 8; i++) {
        bits |= static_cast<std::uint64_t>(bytes[static_cast<std::size_t>(i)]) << (8 * i);
    }
    double out = 0.0;
    std::memcpy(&out, &bits, sizeof(double));
    return out;
}

std::optional<IpcOrigin> originFromTag(std::uint8_t tag) {
    switch (tag) {
        case 1:
            return IpcOrigin::Pulse;
        case 2:
            return IpcOrigin::Signal;
        case 3:
            return IpcOrigin::Composer;
        default:
            return std::nullopt;
    }
}

std::optional<IpcTarget> targetFromTag(std::uint8_t tag) {
    switch (tag) {
        case 1:
            return IpcTarget::Pulse;
        case 2:
            return IpcTarget::Signal;
        case 3:
            return IpcTarget::Composer;
        default:
            return std::nullopt;
    }
}

std::optional<IpcKind> kindFromTag(std::uint8_t tag) {
    switch (tag) {
        case 0:
            return IpcKind::Command;
        case 1:
            return IpcKind::Event;
        case 2:
            return IpcKind::Snapshot;
        case 3:
            return IpcKind::Error;
        default:
            return std::nullopt;
    }
}

std::optional<IpcPriority> priorityFromTag(std::uint8_t tag) {
    switch (tag) {
        case 0:
            return IpcPriority::Realtime;
        case 1:
            return IpcPriority::High;
        case 2:
            return IpcPriority::Normal;
        case 3:
            return IpcPriority::Low;
        default:
            return std::nullopt;
    }
}

nlohmann::json decodeAssetsRegisterAudioAssetPayload(std::span<const std::uint8_t> tlvBytes) {
    TlvReader r(tlvBytes);
    std::optional<std::string> assetId;
    std::optional<std::string> path;
    std::optional<std::uint32_t> channels;
    std::optional<std::uint32_t> sampleRate;
    std::optional<std::uint64_t> frames;

    while (true) {
        auto hOpt = r.readNextHeader();
        if (!hOpt.has_value()) {
            break;
        }

        TlvHeader h = hOpt.value();
        auto valueBytes = r.readValueBytes(h.byteLen);

        if (h.fieldId == 2 && h.fieldType == TLV_STRING) {
            assetId = readTlvString(valueBytes);
            continue;
        }

        if (h.fieldId == 3 && h.fieldType == TLV_STRING) {
            path = readTlvString(valueBytes);
            continue;
        }

        if (h.fieldId == 4 && h.fieldType == TLV_U32) {
            channels = readTlvU32(valueBytes);
            continue;
        }

        if (h.fieldId == 5 && h.fieldType == TLV_U32) {
            sampleRate = readTlvU32(valueBytes);
            continue;
        }

        if (h.fieldId == 6 && h.fieldType == TLV_U64) {
            frames = readTlvU64(valueBytes);
            continue;
        }
    }

    nlohmann::json out = nlohmann::json::object();
    if (assetId.has_value()) {
        out["assetId"] = assetId.value();
    }
    if (path.has_value()) {
        out["path"] = path.value();
    }
    if (channels.has_value()) {
        out["channels"] = channels.value();
    }
    if (sampleRate.has_value()) {
        out["sampleRate"] = sampleRate.value();
    }
    if (frames.has_value()) {
        out["frames"] = frames.value();
    }
    return out;
}

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
                                } else if (mh.fieldId == 3 && mh.fieldType == TLV_F64) {
                                    mix["pan"] = readTlvF64(mv);
                                } else if (mh.fieldId == 4 && mh.fieldType == TLV_BOOL) {
                                    mix["muted"] = readTlvBool(mv);
                                } else if (mh.fieldId == 5 && mh.fieldType == TLV_BOOL) {
                                    mix["soloed"] = readTlvBool(mv);
                                }
                            }
                            node["channelMix"] = mix;
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

std::uint8_t originToTag(IpcOrigin origin) {
    switch (origin) {
        case IpcOrigin::Pulse:
            return 1;
        case IpcOrigin::Signal:
            return 2;
        case IpcOrigin::Composer:
            return 3;
        case IpcOrigin::Aura:
        default:
            return 0; // generic client
    }
}

std::uint8_t targetToTag(IpcTarget target) {
    switch (target) {
        case IpcTarget::Pulse:
            return 1;
        case IpcTarget::Signal:
            return 2;
        case IpcTarget::Composer:
            return 3;
        case IpcTarget::Aura:
        default:
            return 0; // generic client
    }
}

std::uint8_t kindToTag(IpcKind kind) {
    switch (kind) {
        case IpcKind::Command:
            return 0;
        case IpcKind::Event:
            return 1;
        case IpcKind::Snapshot:
            return 2;
        case IpcKind::Error:
            return 3;
    }
    return 0;
}

std::uint8_t priorityToTag(IpcPriority priority) {
    switch (priority) {
        case IpcPriority::Realtime:
            return 0;
        case IpcPriority::High:
            return 1;
        case IpcPriority::Normal:
            return 2;
        case IpcPriority::Low:
            return 3;
    }
    return 2;
}

std::optional<std::vector<std::uint8_t>> encodeEngineStatePayloadTlv(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter w;
        w.writeU32(1, 1);

        if (payload.contains("lifecycle") && payload["lifecycle"].is_string()) {
            w.writeString(2, payload["lifecycle"].get<std::string>());
        }

        // Pulse computes playback-ready separately; Signal doesn't currently include playbackReady.

        if (payload.contains("lastError") && payload["lastError"].is_string()) {
            w.writeString(4, payload["lastError"].get<std::string>());
        }

        if (payload.contains("sampleRate") && payload["sampleRate"].is_number()) {
            w.writeU32(5, static_cast<std::uint32_t>(payload["sampleRate"].get<double>()));
        }

        if (payload.contains("blockSize") && payload["blockSize"].is_number()) {
            w.writeU32(6, static_cast<std::uint32_t>(payload["blockSize"].get<std::uint64_t>()));
        }

        if (payload.contains("numOutputChannels") && payload["numOutputChannels"].is_number()) {
            w.writeU32(7, static_cast<std::uint32_t>(payload["numOutputChannels"].get<std::uint64_t>()));
        }

        if (payload.contains("outputDeviceName") && payload["outputDeviceName"].is_string()) {
            w.writeString(9, payload["outputDeviceName"].get<std::string>());
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodeTransportStatePayloadTlv(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        if (!payload.contains("isPlaying") || !payload["isPlaying"].is_boolean()) {
            error = "transport.state missing isPlaying";
            return std::nullopt;
        }
        if (!payload.contains("positionBeats") || !payload["positionBeats"].is_number()) {
            error = "transport.state missing positionBeats";
            return std::nullopt;
        }
        if (!payload.contains("loopEnabled") || !payload["loopEnabled"].is_boolean()) {
            error = "transport.state missing loopEnabled";
            return std::nullopt;
        }

        TlvWriter w;
        w.writeU32(1, 1);
        w.writeBool(2, payload["isPlaying"].get<bool>());
        w.writeF64(3, payload["positionBeats"].get<double>());
        w.writeBool(4, payload["loopEnabled"].get<bool>());

        if (payload.contains("loopRegion") && payload["loopRegion"].is_object()) {
            const auto& lr = payload["loopRegion"];
            if (lr.contains("startBeats") && lr.contains("endBeats") && lr["startBeats"].is_number() && lr["endBeats"].is_number()) {
                w.writeObject(5, [&](TlvWriter& ww) {
                    ww.writeU32(1, 1);
                    ww.writeF64(2, lr["startBeats"].get<double>());
                    ww.writeF64(3, lr["endBeats"].get<double>());
                });
            }
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodeTransportPositionUpdatePayloadTlv(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        if (!payload.contains("positionSamples") || !payload["positionSamples"].is_number_unsigned()) {
            error = "transport.positionUpdate missing positionSamples";
            return std::nullopt;
        }

        TlvWriter w;
        w.writeU32(1, 1);
        w.writeU64(2, payload["positionSamples"].get<std::uint64_t>());

        // Echo codec treats sampleRate as optional f64.
        if (payload.contains("sampleRate") && payload["sampleRate"].is_number()) {
            w.writeF64(3, payload["sampleRate"].get<double>());
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodeEngineSelfTestResultPayloadTlv(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter w;
        w.writeU32(1, 1);

        if (payload.contains("ok") && payload["ok"].is_boolean()) {
            w.writeBool(2, payload["ok"].get<bool>());
        }

        auto toU32 = [](const nlohmann::json& v) -> std::optional<std::uint32_t> {
            if (v.is_number_unsigned()) {
                return v.get<std::uint32_t>();
            }
            if (v.is_number_integer()) {
                auto i = v.get<std::int64_t>();
                if (i >= 0 && i <= static_cast<std::int64_t>(std::numeric_limits<std::uint32_t>::max())) {
                    return static_cast<std::uint32_t>(i);
                }
                return std::nullopt;
            }
            return std::nullopt;
        };

        if (payload.contains("scenarioCount")) {
            auto v = toU32(payload["scenarioCount"]);
            if (v.has_value()) {
                w.writeU32(3, v.value());
            }
        }

        if (payload.contains("failedScenarioCount")) {
            auto v = toU32(payload["failedScenarioCount"]);
            if (v.has_value()) {
                w.writeU32(4, v.value());
            }
        }

        if (payload.contains("scenarios") && payload["scenarios"].is_array()) {
            std::vector<std::vector<std::uint8_t>> elements;
            for (const auto& sc : payload["scenarios"]) {
                if (!sc.is_object()) {
                    continue;
                }

                TlvWriter sw;
                if (sc.contains("id") && sc["id"].is_string()) {
                    sw.writeString(2, sc["id"].get<std::string>());
                }
                if (sc.contains("ok") && sc["ok"].is_boolean()) {
                    sw.writeBool(3, sc["ok"].get<bool>());
                }
                if (sc.contains("maxAbsSample") && sc["maxAbsSample"].is_number()) {
                    sw.writeF64(4, sc["maxAbsSample"].get<double>());
                }
                elements.push_back(sw.intoBytes());
            }

            if (!elements.empty()) {
                w.writeObjectList(5, elements);
            }
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodeHardwareStatePayloadTlv(
    const nlohmann::json& payload,
    std::string& error
) {
    try {
        TlvWriter w;
        w.writeU32(1, 1);

        if (payload.contains("outputDevices") && payload["outputDevices"].is_array()) {
            std::vector<std::vector<std::uint8_t>> elements;
            for (const auto& dev : payload["outputDevices"]) {
                if (!dev.is_object()) {
                    continue;
                }

                TlvWriter dw;
                if (dev.contains("id") && dev["id"].is_string()) {
                    dw.writeString(2, dev["id"].get<std::string>());
                }
                if (dev.contains("name") && dev["name"].is_string()) {
                    dw.writeString(3, dev["name"].get<std::string>());
                }
                if (dev.contains("isDefault") && dev["isDefault"].is_boolean()) {
                    dw.writeBool(4, dev["isDefault"].get<bool>());
                }
                if (dev.contains("isActive") && dev["isActive"].is_boolean()) {
                    dw.writeBool(5, dev["isActive"].get<bool>());
                }
                if (dev.contains("maxChannels") && dev["maxChannels"].is_number_unsigned()) {
                    dw.writeU32(6, dev["maxChannels"].get<std::uint32_t>());
                }
                if (dev.contains("preferredSampleRate") && dev["preferredSampleRate"].is_number_unsigned()) {
                    dw.writeU32(7, dev["preferredSampleRate"].get<std::uint32_t>());
                }
                elements.push_back(dw.intoBytes());
            }

            if (!elements.empty()) {
                w.writeObjectList(2, elements);
            }
        }

        if (payload.contains("activeDeviceId") && payload["activeDeviceId"].is_string()) {
            w.writeString(3, payload["activeDeviceId"].get<std::string>());
        }
        if (payload.contains("preferredDeviceId") && payload["preferredDeviceId"].is_string()) {
            w.writeString(4, payload["preferredDeviceId"].get<std::string>());
        }
        if (payload.contains("lastError") && payload["lastError"].is_string()) {
            w.writeString(5, payload["lastError"].get<std::string>());
        }

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

} // namespace

std::optional<IpcEnvelope> decodeBinaryEnvelopeV2(
    std::span<const std::uint8_t> bytes,
    std::string& error
) {
    try {
        BinaryReader r(bytes);

        std::uint16_t binaryEnvelopeVersion = r.readU16Le("binaryEnvelopeVersion");
        if (binaryEnvelopeVersion != 2) {
            error = "Unsupported binaryEnvelopeVersion: " + std::to_string(binaryEnvelopeVersion);
            return std::nullopt;
        }

        std::uint16_t envelopeVersion = r.readU16Le("envelopeVersion");
        if (envelopeVersion != 1) {
            error = "Unsupported envelopeVersion: " + std::to_string(envelopeVersion);
            return std::nullopt;
        }

        std::string id = r.readStringU16Len("id");
        std::optional<std::string> cid = r.readOptionalStringU16Len("cid");
        std::string ts = r.readStringU16Len("ts");

        std::uint8_t originTag = r.readU8("originTag");
        std::uint8_t targetTag = r.readU8("targetTag");
        std::uint8_t kindTag = r.readU8("kindTag");

        std::string domain = r.readStringU16Len("domain");
        std::string name = r.readStringU16Len("name");
        std::uint8_t priorityTag = r.readU8("priorityTag");

        std::uint32_t payloadLen = r.readU32Le("payloadLen");
        auto payloadBytes = r.readSlice(payloadLen, "payload");

        std::uint8_t errorTag = r.readU8("errorTag");
        if (errorTag == 1) {
            std::uint32_t errLen = r.readU32Le("errorLen");
            (void)r.readSlice(errLen, "errorBytes");
        } else if (errorTag != 0) {
            error = "Invalid errorTag: " + std::to_string(errorTag);
            return std::nullopt;
        }

        if (r.remaining() != 0) {
            error = "Invalid binary envelope: trailing bytes";
            return std::nullopt;
        }

        auto originOpt = originFromTag(originTag);
        auto targetOpt = targetFromTag(targetTag);
        auto kindOpt = kindFromTag(kindTag);
        auto priorityOpt = priorityFromTag(priorityTag);

        if (!originOpt.has_value()) {
            error = "Invalid originTag: " + std::to_string(originTag);
            return std::nullopt;
        }
        if (!targetOpt.has_value()) {
            error = "Invalid targetTag: " + std::to_string(targetTag);
            return std::nullopt;
        }
        if (!kindOpt.has_value()) {
            error = "Invalid kindTag: " + std::to_string(kindTag);
            return std::nullopt;
        }
        if (!priorityOpt.has_value()) {
            error = "Invalid priorityTag: " + std::to_string(priorityTag);
            return std::nullopt;
        }

        nlohmann::json payload = nlohmann::json::object();

        if (domain == "assets" && name == "registerAudioAsset" && kindOpt.value() == IpcKind::Command) {
            payload = decodeAssetsRegisterAudioAssetPayload(payloadBytes);
        } else if (
            domain == "engine"
            && name == "graphSnapshot"
            && kindOpt.value() == IpcKind::Command
        ) {
            auto decodedPayload = decodeGraphSnapshotPayloadTlv(payloadBytes, error);
            if (!decodedPayload.has_value()) {
                return std::nullopt;
            }
            payload = decodedPayload.value();
        } else if (
            domain == "engine"
            && name == "playbackScheduleSnapshot"
            && kindOpt.value() == IpcKind::Command
        ) {
            auto decodedPayload = decodePlaybackSchedulePayloadTlv(payloadBytes, error);
            if (!decodedPayload.has_value()) {
                return std::nullopt;
            }
            payload = decodedPayload.value();
        } else if (
            domain == "automation"
            && name == "automationSnapshot"
            && kindOpt.value() == IpcKind::Command
        ) {
            auto decodedPayload = decodeAutomationSnapshotPayloadTlv(payloadBytes, error);
            if (!decodedPayload.has_value()) {
                return std::nullopt;
            }
            payload = decodedPayload.value();
        } else if (
            domain == "channelMix"
            && name == "updateChannel"
            && kindOpt.value() == IpcKind::Command
        ) {
            TlvReader rr(payloadBytes);
            std::optional<std::string> channelId;
            std::optional<double> gain;
            std::optional<double> pan;
            std::optional<bool> isMuted;
            std::optional<bool> isSoloed;
            std::optional<bool> effectiveMuted;

            while (true) {
                auto hOpt = rr.readNextHeader();
                if (!hOpt.has_value()) {
                    break;
                }

                TlvHeader h = hOpt.value();
                auto valueBytes = rr.readValueBytes(h.byteLen);

                if (h.fieldId == 2 && h.fieldType == TLV_STRING) {
                    channelId = readTlvString(valueBytes);
                } else if (h.fieldId == 3 && h.fieldType == TLV_F64) {
                    gain = readTlvF64(valueBytes);
                } else if (h.fieldId == 4 && h.fieldType == TLV_F64) {
                    pan = readTlvF64(valueBytes);
                } else if (h.fieldId == 5 && h.fieldType == TLV_BOOL) {
                    isMuted = readTlvBool(valueBytes);
                } else if (h.fieldId == 6 && h.fieldType == TLV_BOOL) {
                    isSoloed = readTlvBool(valueBytes);
                } else if (h.fieldId == 7 && h.fieldType == TLV_BOOL) {
                    effectiveMuted = readTlvBool(valueBytes);
                }
            }

            if (!channelId.has_value()) {
                error = "channelMix.updateChannel missing channelId";
                return std::nullopt;
            }
            if (!gain.has_value()) {
                error = "channelMix.updateChannel missing gain";
                return std::nullopt;
            }
            if (!pan.has_value()) {
                error = "channelMix.updateChannel missing pan";
                return std::nullopt;
            }
            if (!isMuted.has_value()) {
                error = "channelMix.updateChannel missing isMuted";
                return std::nullopt;
            }
            if (!isSoloed.has_value()) {
                error = "channelMix.updateChannel missing isSoloed";
                return std::nullopt;
            }
            if (!effectiveMuted.has_value()) {
                error = "channelMix.updateChannel missing effectiveMuted";
                return std::nullopt;
            }

            payload["channelId"] = channelId.value();
            payload["gain"] = gain.value();
            payload["pan"] = pan.value();
            payload["isMuted"] = isMuted.value();
            payload["isSoloed"] = isSoloed.value();
            payload["effectiveMuted"] = effectiveMuted.value();
        } else if (
            domain == "node"
            && name == "setParameter"
            && kindOpt.value() == IpcKind::Command
        ) {
            TlvReader rr(payloadBytes);
            std::optional<std::string> nodeId;
            std::optional<std::string> parameterId;
            std::optional<double> value;

            while (true) {
                auto hOpt = rr.readNextHeader();
                if (!hOpt.has_value()) {
                    break;
                }

                TlvHeader h = hOpt.value();
                auto valueBytes = rr.readValueBytes(h.byteLen);

                if (h.fieldId == 2 && h.fieldType == TLV_STRING) {
                    nodeId = readTlvString(valueBytes);
                } else if (h.fieldId == 3 && h.fieldType == TLV_STRING) {
                    parameterId = readTlvString(valueBytes);
                } else if (h.fieldId == 4 && h.fieldType == TLV_F64) {
                    value = readTlvF64(valueBytes);
                }
            }

            payload["nodeId"] = nodeId.value_or("");
            payload["parameterId"] = parameterId.value_or("");
            payload["value"] = value.value_or(0.0);
        } else if (
            domain == "hardware"
            && name == "refreshOutputDevices"
            && kindOpt.value() == IpcKind::Command
        ) {
            // Schema-only TLV payload; no fields required.
            payload = nlohmann::json::object();
        } else if (
            domain == "hardware"
            && name == "selectOutputDevice"
            && kindOpt.value() == IpcKind::Command
        ) {
            TlvReader rr(payloadBytes);
            std::optional<std::string> id;

            while (true) {
                auto hOpt = rr.readNextHeader();
                if (!hOpt.has_value()) {
                    break;
                }

                TlvHeader h = hOpt.value();
                auto valueBytes = rr.readValueBytes(h.byteLen);

                if (h.fieldId == 2 && h.fieldType == TLV_STRING) {
                    id = readTlvString(valueBytes);
                }
            }

            payload["id"] = id.value_or("");
        } else if (domain == "transport" && name == "play" && kindOpt.value() == IpcKind::Command) {
            // Schema-only TLV payload; no fields required.
            payload = nlohmann::json::object();
        } else if (domain == "transport" && name == "stop" && kindOpt.value() == IpcKind::Command) {
            TlvReader rr(payloadBytes);
            std::optional<double> positionBeats;

            while (true) {
                auto hOpt = rr.readNextHeader();
                if (!hOpt.has_value()) {
                    break;
                }

                TlvHeader h = hOpt.value();
                auto valueBytes = rr.readValueBytes(h.byteLen);

                if (h.fieldId == 2 && h.fieldType == TLV_F64) {
                    positionBeats = readTlvF64(valueBytes);
                }
            }

            if (positionBeats.has_value()) {
                payload["positionBeats"] = positionBeats.value();
            }
        } else if (domain == "transport" && name == "seek" && kindOpt.value() == IpcKind::Command) {
            TlvReader rr(payloadBytes);
            std::optional<double> positionBeats;

            while (true) {
                auto hOpt = rr.readNextHeader();
                if (!hOpt.has_value()) {
                    break;
                }

                TlvHeader h = hOpt.value();
                auto valueBytes = rr.readValueBytes(h.byteLen);

                if (h.fieldId == 2 && h.fieldType == TLV_F64) {
                    positionBeats = readTlvF64(valueBytes);
                }
            }

            if (!positionBeats.has_value()) {
                error = "transport.seek missing positionBeats";
                return std::nullopt;
            }
            payload["positionBeats"] = positionBeats.value();
        } else if (
            domain == "transport"
            && name == "setLoopEnabled"
            && kindOpt.value() == IpcKind::Command
        ) {
            TlvReader rr(payloadBytes);
            std::optional<bool> enabled;

            while (true) {
                auto hOpt = rr.readNextHeader();
                if (!hOpt.has_value()) {
                    break;
                }

                TlvHeader h = hOpt.value();
                auto valueBytes = rr.readValueBytes(h.byteLen);

                if (h.fieldId == 2 && h.fieldType == TLV_BOOL) {
                    enabled = readTlvBool(valueBytes);
                }
            }

            if (!enabled.has_value()) {
                error = "transport.setLoopEnabled missing enabled";
                return std::nullopt;
            }
            payload["enabled"] = enabled.value();
        } else if (
            domain == "transport"
            && name == "setLoopRegion"
            && kindOpt.value() == IpcKind::Command
        ) {
            TlvReader rr(payloadBytes);
            std::optional<double> startBeats;
            std::optional<double> endBeats;

            while (true) {
                auto hOpt = rr.readNextHeader();
                if (!hOpt.has_value()) {
                    break;
                }

                TlvHeader h = hOpt.value();
                auto valueBytes = rr.readValueBytes(h.byteLen);

                if (h.fieldId == 2 && h.fieldType == TLV_F64) {
                    startBeats = readTlvF64(valueBytes);
                } else if (h.fieldId == 3 && h.fieldType == TLV_F64) {
                    endBeats = readTlvF64(valueBytes);
                }
            }

            if (!startBeats.has_value()) {
                error = "transport.setLoopRegion missing startBeats";
                return std::nullopt;
            }
            if (!endBeats.has_value()) {
                error = "transport.setLoopRegion missing endBeats";
                return std::nullopt;
            }

            payload["startBeats"] = startBeats.value();
            payload["endBeats"] = endBeats.value();
        } else if (domain == "engine" && name == "state" && kindOpt.value() == IpcKind::Event) {
            TlvReader rr(payloadBytes);
            while (true) {
                auto hOpt = rr.readNextHeader();
                if (!hOpt.has_value()) {
                    break;
                }
                TlvHeader h = hOpt.value();
                auto valueBytes = rr.readValueBytes(h.byteLen);
                if (h.fieldId == 2 && h.fieldType == TLV_STRING) {
                    payload["lifecycle"] = readTlvString(valueBytes);
                } else if (h.fieldId == 5 && h.fieldType == TLV_U32) {
                    payload["sampleRate"] = readTlvU32(valueBytes);
                } else if (h.fieldId == 6 && h.fieldType == TLV_U32) {
                    payload["bufferSize"] = readTlvU32(valueBytes);
                } else if (h.fieldId == 7 && h.fieldType == TLV_U32) {
                    payload["numOutputChannels"] = readTlvU32(valueBytes);
                } else if (h.fieldId == 9 && h.fieldType == TLV_STRING) {
                    payload["outputDeviceName"] = readTlvString(valueBytes);
                } else if (h.fieldId == 4 && h.fieldType == TLV_STRING) {
                    payload["lastError"] = readTlvString(valueBytes);
                }
            }
        } else if (
            domain == "engine"
            && (
                name == "start"
                || name == "stop"
                || name == "reset"
                || name == "shutdown"
                || name == "heartbeat"
                || name == "selfTest"
            )
            && kindOpt.value() == IpcKind::Command
        ) {
            // Schema-only TLV payload; no fields required.
            payload = nlohmann::json::object();
        }

        IpcEnvelope out;
        out.version = 1;
        out.id = id;
        out.correlationId = cid;
        out.timestamp = ts;
        out.origin = originOpt.value();
        out.target = targetOpt.value();
        out.domain = domain;
        out.kind = kindOpt.value();
        out.name = name;
        out.priority = priorityOpt.value();
        out.payload = payload;
        out.error = std::nullopt;

        return out;
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> encodeBinaryEnvelopeV2(
    const IpcEnvelope& envelope,
    std::span<const std::uint8_t> payloadTlvBytes,
    std::string& error
) {
    try {
        BinaryWriter w;
        w.writeU16Le(2); // binaryEnvelopeVersion
        w.writeU16Le(1); // envelopeVersion

        w.writeStringU16Len(envelope.id);
        w.writeOptionalStringU16Len(envelope.correlationId);
        w.writeStringU16Len(envelope.timestamp);

        w.writeU8(originToTag(envelope.origin));
        w.writeU8(targetToTag(envelope.target));
        w.writeU8(kindToTag(envelope.kind));

        w.writeStringU16Len(envelope.domain);
        w.writeStringU16Len(envelope.name);

        w.writeU8(priorityToTag(envelope.priority));

        if (payloadTlvBytes.size() > 0xffff'ffff) {
            error = "Payload too large for u32 length";
            return std::nullopt;
        }
        w.writeU32Le(static_cast<std::uint32_t>(payloadTlvBytes.size()));
        w.writeBytes(payloadTlvBytes);

        // Error payload encoding not implemented yet (write absent).
        w.writeU8(0);

        return w.intoBytes();
    } catch (const std::exception& e) {
        error = e.what();
        return std::nullopt;
    }
}

std::optional<std::vector<std::uint8_t>> tryEncodeBinaryEnvelopeV2(
    const IpcEnvelope& envelope,
    std::string& error
) {
    if (envelope.domain == "engine" && envelope.name == "state" && envelope.kind == IpcKind::Event) {
        auto payload = encodeEngineStatePayloadTlv(envelope.payload, error);
        if (!payload.has_value()) {
            return std::nullopt;
        }

        return encodeBinaryEnvelopeV2(envelope, payload.value(), error);
    }

    if (envelope.domain == "transport" && envelope.name == "state" && envelope.kind == IpcKind::Event) {
        auto payload = encodeTransportStatePayloadTlv(envelope.payload, error);
        if (!payload.has_value()) {
            return std::nullopt;
        }
        return encodeBinaryEnvelopeV2(envelope, payload.value(), error);
    }

    if (
        envelope.domain == "transport"
        && envelope.name == "positionUpdate"
        && envelope.kind == IpcKind::Event
    ) {
        auto payload = encodeTransportPositionUpdatePayloadTlv(envelope.payload, error);
        if (!payload.has_value()) {
            return std::nullopt;
        }
        return encodeBinaryEnvelopeV2(envelope, payload.value(), error);
    }

    if (
        envelope.domain == "engine"
        && envelope.name == "selfTestResult"
        && envelope.kind == IpcKind::Event
    ) {
        auto payload = encodeEngineSelfTestResultPayloadTlv(envelope.payload, error);
        if (!payload.has_value()) {
            return std::nullopt;
        }
        return encodeBinaryEnvelopeV2(envelope, payload.value(), error);
    }

    if (envelope.domain == "hardware" && envelope.name == "state" && envelope.kind == IpcKind::Event) {
        auto payload = encodeHardwareStatePayloadTlv(envelope.payload, error);
        if (!payload.has_value()) {
            return std::nullopt;
        }
        return encodeBinaryEnvelopeV2(envelope, payload.value(), error);
    }

    return std::nullopt;
}

} // namespace loophole::signal::ipc
