#include "core/SignalApp.hpp"
#include "ipc/Router.hpp"
#include "ipc/TcpServer.hpp"
#include "ipc/DomainDispatcher.hpp"
#include "core/EngineHost.hpp"
#include "domains/EngineDomain.hpp"
#include "domains/TransportDomain.hpp"
#include <asio/io_context.hpp>
#include <asio/signal_set.hpp>
#include <cstdlib>
#include <iostream>
#include <signal.h>
#include <thread>

SignalApp::SignalApp() {
    std::cout << "[SignalApp] Initialising..." << std::endl;

    _engineHost = std::make_unique<EngineHost>();
    _router = std::make_unique<IpcRouter>();

    // Register domain handlers
    auto engineDomain = std::make_shared<EngineDomain>(_engineHost.get());
    _router->registerHandler("engine", engineDomain);

    auto transportDomain = std::make_shared<TransportDomain>(_engineHost.get());
    _router->registerHandler("transport", transportDomain);

    std::cout << "[SignalApp] Initialised" << std::endl;
}

SignalApp::~SignalApp() {
    std::cout << "[SignalApp] Shutting down..." << std::endl;
}

int SignalApp::run() {
    std::cout << "[SignalApp] Running..." << std::endl;

    // Get host/port from environment or use defaults
    std::string host = "127.0.0.1";
    uint16_t port = 7888;

    const char* host_env = std::getenv("SIGNAL_HOST");
    if (host_env != nullptr) {
        host = host_env;
    }

    const char* port_env = std::getenv("SIGNAL_PORT");
    if (port_env != nullptr) {
        try {
            port = static_cast<uint16_t>(std::stoi(port_env));
        } catch (const std::exception& e) {
            std::cerr << "[SignalApp] Invalid SIGNAL_PORT, using default 7888" << std::endl;
        }
    }

    // Create IO context and server
    asio::io_context io;
    loophole::signal::ipc::DomainDispatcher dispatcher(_router.get(), _engineHost.get());

    loophole::signal::ipc::TcpServer server(
        io,
        host,
        port,
        [&dispatcher](
            const loophole::signal::ipc::IpcEnvelope& env,
            std::shared_ptr<loophole::signal::ipc::TcpClientSession> session
        ) {
            dispatcher.handleEnvelope(env, session);
        }
    );

    // Set up signal handling for graceful shutdown
    asio::signal_set signals(io, SIGINT, SIGTERM);
    signals.async_wait(
        [&server, &io](std::error_code /*ec*/, int /*signo*/) {
            std::cout << "[SignalApp] Shutdown signal received" << std::endl;
            server.stop();
            io.stop();
        }
    );

    // Start server
    server.start();

    // Run IO loop
    std::cout << "[SignalApp] Starting IO loop..." << std::endl;
    io.run();

    std::cout << "[SignalApp] IO loop finished" << std::endl;
    return 0;
}

