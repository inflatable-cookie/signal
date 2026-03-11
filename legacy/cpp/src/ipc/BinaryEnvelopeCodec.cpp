#include "ipc/BinaryEnvelopeCodec.hpp"
#include "ipc/binary_envelope/CodecCommon.hpp"
#include "ipc/binary_envelope/PayloadCodecs.hpp"
#include <nlohmann/json.hpp>

namespace loophole::signal::ipc {

std::optional<IpcEnvelope> decodeBinaryEnvelopeV2(
    std::span<const std::uint8_t> bytes,
    std::string& error
) {
    try {
        using namespace binary_envelope;

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

        auto decodedPayload = binary_envelope::decodeTypedPayload(
            domain,
            name,
            kindOpt.value(),
            payloadBytes,
            error
        );
        if (decodedPayload.has_value()) {
            payload = decodedPayload.value();
        } else if (!error.empty()) {
            return std::nullopt;
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
        using namespace binary_envelope;

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
    error.clear();

    auto payloadBytes = binary_envelope::encodeTypedPayload(
        envelope.domain,
        envelope.name,
        envelope.kind,
        envelope.payload,
        error
    );
    if (!payloadBytes.has_value()) {
        error = "No typed payload codec for " + envelope.domain + "." + envelope.name + " (" + kindToString(envelope.kind) + ")";
        return std::nullopt;
    }

    return encodeBinaryEnvelopeV2(envelope, payloadBytes.value(), error);
}

} // namespace loophole::signal::ipc
