#include "ipc/TcpClientSession.hpp"
#include "ipc/BinaryEnvelopeCodec.hpp"
#include "logging/Logging.hpp"
#include <asio/buffer.hpp>
#include <asio/read_until.hpp>
#include <asio/write.hpp>
#include <sstream>

namespace loophole::signal::ipc {

TcpClientSession::TcpClientSession(
    asio::ip::tcp::socket socket,
    EnvelopeHandler handler
) : socket_(std::move(socket)), handler_(std::move(handler)) {
}

void TcpClientSession::start() {
    doRead();
}

void TcpClientSession::doRead() {
    // Signal control-plane IPC is framed-only (LPF1) and binary-envelope-v2 only.
    doReadBinary();
}

void TcpClientSession::doReadBinary() {
    auto self = shared_from_this();

    socket_.async_read_some(
        asio::buffer(binaryReadChunk_),
        [this, self](std::error_code ec, std::size_t bytesRead) {
            if (ec) {
                if (ec == asio::error::eof || ec == asio::error::connection_reset) {
                    LOG_DEBUG({"TcpClientSession"}, "Client disconnected");
                } else {
                    LOG_ERROR({"TcpClientSession"}, std::string("Read error: ") + ec.message());
                }

                if (disconnectedCallback_) {
                    disconnectedCallback_();
                }

                return;
            }

            if (bytesRead > 0) {
                handleBinaryBytes(std::span<const std::uint8_t>(binaryReadChunk_.data(), bytesRead));
            }

            doReadBinary();
        }
    );
}

void TcpClientSession::handleBinaryBytes(std::span<const std::uint8_t> bytes) {
    binaryBuffer_.insert(binaryBuffer_.end(), bytes.begin(), bytes.end());
    processBinaryBuffer();
}

void TcpClientSession::processBinaryBuffer() {
    constexpr std::uint8_t expectedMagic[4] = {'L', 'P', 'F', '1'};
    constexpr std::size_t maxFrameLen = 16 * 1024 * 1024;
    constexpr std::uint8_t kindBinaryEnvelope = 3;

    if (!binaryMagicConsumed_) {
        if (binaryBuffer_.size() < 4) {
            return;
        }

        if (std::memcmp(binaryBuffer_.data(), expectedMagic, 4) != 0) {
            LOG_ERROR({"TcpClientSession"}, "Invalid LPF1 magic on control-plane connection");
            close();
            return;
        }

        binaryMagicConsumed_ = true;
        binaryBuffer_.erase(binaryBuffer_.begin(), binaryBuffer_.begin() + 4);
    }

    while (binaryBuffer_.size() >= 4) {
        std::uint32_t len = 0;
        len |= static_cast<std::uint32_t>(binaryBuffer_[0]);
        len |= static_cast<std::uint32_t>(binaryBuffer_[1]) << 8;
        len |= static_cast<std::uint32_t>(binaryBuffer_[2]) << 16;
        len |= static_cast<std::uint32_t>(binaryBuffer_[3]) << 24;

        if (len == 0 || len > maxFrameLen) {
            LOG_ERROR({"TcpClientSession"}, std::string("Invalid framed-binary length: ") + std::to_string(len));
            close();
            return;
        }

        if (binaryBuffer_.size() < 4 + len) {
            break;
        }

        std::vector<std::uint8_t> frame(binaryBuffer_.begin() + 4, binaryBuffer_.begin() + 4 + len);
        binaryBuffer_.erase(binaryBuffer_.begin(), binaryBuffer_.begin() + 4 + len);

        if (frame.empty()) {
            continue;
        }

        std::uint8_t kind = frame[0];
        if (kind != kindBinaryEnvelope) {
            LOG_ERROR({"TcpClientSession"}, std::string("Unsupported framed-binary kind: ") + std::to_string(kind));
            close();
            return;
        }

        std::string err;
        auto envOpt = decodeBinaryEnvelopeV2(
            std::span<const std::uint8_t>(frame.data() + 1, frame.size() - 1),
            err
        );

        if (!envOpt.has_value()) {
            LOG_WARN({"TcpClientSession"}, std::string("Failed to decode binary envelope: ") + err);
            continue;
        }

        handler_(envOpt.value(), shared_from_this());
    }
}

void TcpClientSession::send(const IpcEnvelope& env) {
    std::lock_guard<std::mutex> lock(writeMutex_);

    try {
        constexpr std::uint8_t kindBinaryEnvelope = 3;

        if (!framedMagicSent_) {
            const char magic[4] = {'L', 'P', 'F', '1'};
            asio::write(socket_, asio::buffer(magic, 4));
            framedMagicSent_ = true;
        }

        std::string err;
        auto bin = tryEncodeBinaryEnvelopeV2(env, err);
        if (!bin.has_value()) {
            LOG_WARN(
                {"TcpClientSession"},
                std::string("Dropping outbound envelope (binary codec missing): ")
                    + env.domain + "." + env.name + " (" + kindToString(env.kind) + "): " + err
            );
            return;
        }

        std::uint32_t len = static_cast<std::uint32_t>(1 + bin.value().size());
        std::uint8_t lenBytes[4] = {
            static_cast<std::uint8_t>(len & 0xff),
            static_cast<std::uint8_t>((len >> 8) & 0xff),
            static_cast<std::uint8_t>((len >> 16) & 0xff),
            static_cast<std::uint8_t>((len >> 24) & 0xff),
        };

        asio::write(socket_, asio::buffer(lenBytes, 4));
        asio::write(socket_, asio::buffer(&kindBinaryEnvelope, 1));
        asio::write(socket_, asio::buffer(bin.value()));
    } catch (const std::exception& e) {
        LOG_ERROR({"TcpClientSession"}, std::string("Send error: ") + e.what());
    }
}

void TcpClientSession::close() {
    std::lock_guard<std::mutex> lock(writeMutex_);

    if (socket_.is_open()) {
        std::error_code ec;
        socket_.close(ec);
        // Notify disconnect callback
        if (disconnectedCallback_) {
            disconnectedCallback_();
        }
    }
}

void TcpClientSession::setDisconnectedCallback(DisconnectedCallback callback) {
    disconnectedCallback_ = std::move(callback);
}

} // namespace loophole::signal::ipc
