#include "ipc/TcpServer.hpp"
#include <asio/bind_executor.hpp>
#include <asio/ip/tcp.hpp>
#include <iostream>

namespace loophole::signal::ipc {

TcpServer::TcpServer(
    asio::io_context& io,
    const std::string& host,
    uint16_t port,
    EnvelopeHandler handler
) : io_(io), acceptor_(io), handler_(std::move(handler)) {
    std::error_code ec;
    asio::ip::tcp::endpoint endpoint(asio::ip::address::from_string(host, ec), port);
    if (ec) {
        std::cerr << "[TcpServer] Invalid host address: " << host << std::endl;
        throw std::runtime_error("Invalid host address");
    }
    acceptor_.open(endpoint.protocol());
    acceptor_.set_option(asio::ip::tcp::acceptor::reuse_address(true));
    acceptor_.bind(endpoint);
    acceptor_.listen();
}

void TcpServer::start() {
    std::string host = acceptor_.local_endpoint().address().to_string();
    uint16_t port = acceptor_.local_endpoint().port();
    std::cout << "[Signal] IPC server listening on " << host << ":" << port << std::endl;

    doAccept();
}

void TcpServer::doAccept() {
    acceptor_.async_accept(
        [this](std::error_code ec, asio::ip::tcp::socket socket) {
            if (!ec) {
                std::error_code ep_ec;
                auto remote_ep = socket.remote_endpoint(ep_ec);
                if (!ep_ec) {
                    std::cout << "[TcpServer] Client connected from "
                              << remote_ep.address().to_string()
                              << ":" << remote_ep.port() << std::endl;
                }

                auto session = std::make_shared<TcpClientSession>(
                    std::move(socket),
                    handler_
                );

                {
                    std::lock_guard<std::mutex> lock(clientsMutex_);
                    clients_.push_back(std::weak_ptr<TcpClientSession>(session));
                }

                session->start();
            } else {
                std::cerr << "[TcpServer] Accept error: " << ec.message() << std::endl;
            }

            // Continue accepting new connections
            if (acceptor_.is_open()) {
                doAccept();
            }
        }
    );
}

void TcpServer::stop() {
    std::lock_guard<std::mutex> lock(clientsMutex_);

    // Close all active sessions
    for (auto& weak_session : clients_) {
        if (auto session = weak_session.lock()) {
            session->close();
        }
    }

    clients_.clear();

    if (acceptor_.is_open()) {
        acceptor_.close();
        std::cout << "[TcpServer] Server stopped" << std::endl;
    }
}

} // namespace loophole::signal::ipc

