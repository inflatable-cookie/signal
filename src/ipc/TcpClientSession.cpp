#include "ipc/TcpClientSession.hpp"
#include "ipc/IpcEnvelopeCodec.hpp"
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
    auto self = shared_from_this();
    asio::async_read_until(
        socket_,
        asio::dynamic_buffer(readBuffer_),
        '\n',
        [this, self](std::error_code ec, std::size_t /*bytes_read*/) {
            if (ec) {
                if (ec == asio::error::eof || ec == asio::error::connection_reset) {
                    LOG_DEBUG({"TcpClientSession"}, "Client disconnected");
                } else {
                    LOG_ERROR({"TcpClientSession"}, std::string("Read error: ") + ec.message());
                }
                return;
            }

            // Extract complete line(s) from buffer
            std::size_t newline_pos = 0;
            while ((newline_pos = readBuffer_.find('\n')) != std::string::npos) {
                std::string line = readBuffer_.substr(0, newline_pos);
                readBuffer_.erase(0, newline_pos + 1);

                if (!line.empty()) {
                    handleLine(line);
                }
            }

            // Continue reading
            doRead();
        }
    );
}

void TcpClientSession::handleLine(std::string_view line) {
    auto env_opt = deserialiseEnvelope(line);
    if (!env_opt.has_value()) {
        LOG_ERROR({"TcpClientSession"}, "Failed to parse envelope, skipping");
        return;
    }

    handler_(env_opt.value(), shared_from_this());
}

void TcpClientSession::send(const IpcEnvelope& env) {
    std::lock_guard<std::mutex> lock(writeMutex_);

    try {
        std::string json_line = serialiseEnvelope(env) + "\n";
        asio::write(socket_, asio::buffer(json_line));
    } catch (const std::exception& e) {
        LOG_ERROR({"TcpClientSession"}, std::string("Send error: ") + e.what());
    }
}

void TcpClientSession::close() {
    std::lock_guard<std::mutex> lock(writeMutex_);

    if (socket_.is_open()) {
        std::error_code ec;
        socket_.close(ec);
    }
}

} // namespace loophole::signal::ipc

