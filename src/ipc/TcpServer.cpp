/// TcpServer - IPC server for receiving envelopes from Pulse
///
/// Thread: IPC thread (Asio io_context worker threads)
/// Ownership: Owned by SignalApp::run() (local to run method)
/// Communication:
///   - Accepts TCP connections from Pulse
///   - Creates TcpClientSession for each connection
///   - Envelope handlers run in Asio handler context (IPC thread)
///   - Domain handlers update EngineHost/TransportState synchronously

#include "ipc/TcpServer.hpp"
#include "logging/Logging.hpp"
#include <sstream>
#include <asio/bind_executor.hpp>
#include <asio/ip/tcp.hpp>

namespace loophole::signal::ipc {

TcpServer::TcpServer(
    asio::io_context& io,
    const std::string& host,
    uint16_t port,
    EnvelopeHandler handler
) : io_(io), acceptor_(io), handler_(std::move(handler)),
    lastPositionReportTime_(std::chrono::steady_clock::now()),
    lastReportedPositionSamples_(0) {
    std::error_code ec;
    asio::ip::tcp::endpoint endpoint(asio::ip::address::from_string(host, ec), port);
    if (ec) {
        LOG_ERROR({"TcpServer"}, std::string("Invalid host address: ") + host);
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
    std::ostringstream msg;
    msg << "IPC server listening on " << host << ":" << port;
    LOG_INFO({"TcpServer"}, msg.str());

    doAccept();
}

void TcpServer::doAccept() {
    acceptor_.async_accept(
        [this](std::error_code ec, asio::ip::tcp::socket socket) {
            if (!ec) {
                std::error_code ep_ec;
                auto remote_ep = socket.remote_endpoint(ep_ec);
                if (!ep_ec) {
                    std::ostringstream connMsg;
                    connMsg << "Client connected from "
                            << remote_ep.address().to_string()
                            << ":" << remote_ep.port();
                    LOG_INFO({"TcpServer"}, connMsg.str());
                }

                auto session = std::make_shared<TcpClientSession>(
                    std::move(socket),
                    handler_
                );

                {
                    std::lock_guard<std::mutex> lock(clientsMutex_);
                    clients_.push_back(std::weak_ptr<TcpClientSession>(session));
                }

                // Update client tracking
                hasEverSeenClient_.store(true);
                activeClientCount_++;

                // Set up disconnect callback for this session
                auto weak_self = std::weak_ptr<TcpClientSession>(session);
                session->setDisconnectedCallback([this, weak_self]() {
                    // Only decrement if this session was actually active
                    // (avoid double-counting if close() is called multiple times)
                    if (weak_self.lock()) {
                        int prev_count = activeClientCount_.fetch_sub(1);
                        if (prev_count <= 0) {
                            // Clamp to 0 (shouldn't happen, but be safe)
                            activeClientCount_.store(0);
                        }

                        // Call client disconnected callback if set
                        if (clientDisconnectedCallback_) {
                            clientDisconnectedCallback_();
                        }
                    }
                });

                session->start();

                // Call client connected callback if set
                if (clientConnectedCallback_) {
                    clientConnectedCallback_(session);
                }
            } else {
                LOG_ERROR({"TcpServer"}, std::string("Accept error: ") + ec.message());
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
        LOG_INFO({"TcpServer"}, "Server stopped");
    }
}

void TcpServer::setClientConnectedCallback(ClientConnectedCallback callback) {
    clientConnectedCallback_ = std::move(callback);
}

void TcpServer::setClientDisconnectedCallback(ClientDisconnectedCallback callback) {
    clientDisconnectedCallback_ = std::move(callback);
}

int TcpServer::getActiveClientCount() const {
    return activeClientCount_.load();
}

bool TcpServer::hasEverSeenClient() const {
    return hasEverSeenClient_.load();
}

} // namespace loophole::signal::ipc

