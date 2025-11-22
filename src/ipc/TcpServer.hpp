#pragma once

#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include <asio/ip/tcp.hpp>
#include <memory>
#include <mutex>
#include <string>
#include <vector>

namespace loophole::signal::ipc {

/// TCP server for IPC envelope communication
class TcpServer {
public:
    using EnvelopeHandler = TcpClientSession::EnvelopeHandler;

    TcpServer(
        asio::io_context& io,
        const std::string& host,
        uint16_t port,
        EnvelopeHandler handler
    );

    void start();
    void stop();

private:
    void doAccept();

    asio::io_context& io_;
    asio::ip::tcp::acceptor acceptor_;
    EnvelopeHandler handler_;
    std::mutex clientsMutex_;
    std::vector<std::weak_ptr<TcpClientSession>> clients_;
};

} // namespace loophole::signal::ipc

