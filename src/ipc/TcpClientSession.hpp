#pragma once

#include "ipc/IpcEnvelope.hpp"
#include <asio/ip/tcp.hpp>
#include <array>
#include <cstdint>
#include <functional>
#include <memory>
#include <mutex>
#include <span>
#include <string>
#include <vector>

namespace loophole::signal::ipc {

/// TCP client session for handling a single connected client
class TcpClientSession : public std::enable_shared_from_this<TcpClientSession> {
public:
    using EnvelopeHandler = std::function<void(
        const IpcEnvelope&,
        std::shared_ptr<TcpClientSession>
    )>;

    TcpClientSession(
        asio::ip::tcp::socket socket,
        EnvelopeHandler handler
    );

    void start();
    void send(const IpcEnvelope& env);
    void close();

    // Set callback to be called when this session disconnects
    using DisconnectedCallback = std::function<void()>;
    void setDisconnectedCallback(DisconnectedCallback callback);

private:
    void doRead();
    void handleLine(std::string_view line);

    void doReadBinary();
    void handleBinaryBytes(std::span<const std::uint8_t> bytes);
    void processBinaryBuffer();

    asio::ip::tcp::socket socket_;
    EnvelopeHandler handler_;
    std::vector<std::uint8_t> binaryBuffer_;
    std::array<std::uint8_t, 8192> binaryReadChunk_{};
    bool binaryMagicConsumed_ = false;
    bool framedMagicSent_ = false;
    std::mutex writeMutex_;
    DisconnectedCallback disconnectedCallback_;
};

} // namespace loophole::signal::ipc
