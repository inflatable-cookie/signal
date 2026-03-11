#pragma once

#include "ipc/IpcEnvelope.hpp"
#include <cstdint>
#include <cstring>
#include <optional>
#include <span>
#include <stdexcept>
#include <string>
#include <string_view>
#include <vector>

namespace loophole::signal::ipc::binary_envelope {

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

    void writeList(
        std::uint16_t fieldId,
        std::uint8_t elementType,
        const std::vector<std::vector<std::uint8_t>>& elements
    ) {
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

inline std::string readTlvString(std::span<const std::uint8_t> bytes) {
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

inline std::uint32_t readTlvU32(std::span<const std::uint8_t> bytes) {
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

inline bool readTlvBool(std::span<const std::uint8_t> bytes) {
    if (bytes.size() != 1) {
        throw std::runtime_error("Invalid TLV bool: wrong length");
    }

    return bytes[0] != 0;
}

inline std::uint64_t readTlvU64(std::span<const std::uint8_t> bytes) {
    if (bytes.size() != 8) {
        throw std::runtime_error("Invalid TLV u64: wrong length");
    }

    std::uint64_t v = 0;
    for (int i = 0; i < 8; i++) {
        v |= static_cast<std::uint64_t>(bytes[static_cast<std::size_t>(i)]) << (8 * i);
    }
    return v;
}

inline double readTlvF64(std::span<const std::uint8_t> bytes) {
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

inline std::optional<IpcOrigin> originFromTag(std::uint8_t tag) {
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

inline std::optional<IpcTarget> targetFromTag(std::uint8_t tag) {
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

inline std::optional<IpcKind> kindFromTag(std::uint8_t tag) {
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

inline std::optional<IpcPriority> priorityFromTag(std::uint8_t tag) {
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

inline std::uint8_t originToTag(IpcOrigin origin) {
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

inline std::uint8_t targetToTag(IpcTarget target) {
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

inline std::uint8_t kindToTag(IpcKind kind) {
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

inline std::uint8_t priorityToTag(IpcPriority priority) {
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

} // namespace loophole::signal::ipc::binary_envelope
