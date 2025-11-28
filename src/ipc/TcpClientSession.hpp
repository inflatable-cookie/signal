#pragma once

#include "ipc/IpcEnvelope.hpp"
#include <asio/ip/tcp.hpp>
#include <functional>
#include <memory>
#include <mutex>
#include <string>

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

    // Set callback to be called when the client disconnects
    using DisconnectedCallback = std::function<void()>;
    void setDisconnectedCallback(DisconnectedCallback callback);

private:
    void doRead();
    void handleLine(std::string_view line);
    void notifyDisconnected();

    asio::ip::tcp::socket socket_;
    EnvelopeHandler handler_;
    std::string readBuffer_;
    std::mutex writeMutex_;
    DisconnectedCallback disconnectedCallback_;
    std::atomic<bool> disconnectedNotified_{false};
};

} // namespace loophole::signal::ipc

