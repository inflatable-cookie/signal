#include "core/SignalApp.hpp"
#include "ipc/Router.hpp"
#include "ipc/TcpServer.hpp"
#include "ipc/DomainDispatcher.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "core/EngineHost.hpp"
#include "domains/EngineDomain.hpp"
#include "domains/TransportDomain.hpp"
#include "domains/MeteringDomain.hpp"
#include <asio/io_context.hpp>
#include <asio/steady_timer.hpp>
#include <asio/signal_set.hpp>
#include <atomic>
#include <chrono>
#include <cstdlib>
#include <functional>
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

    auto meteringDomain = std::make_shared<MeteringDomain>(&_engineHost->metering(), _engineHost.get());
    _router->registerHandler("metering", meteringDomain);

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
    std::atomic<bool> shuttingDown{false};

    signals.async_wait(
        [&server, &io, this, &shuttingDown](std::error_code /*ec*/, int /*signo*/) {
            std::cout << "[SignalApp] Shutdown signal received" << std::endl;
            shuttingDown.store(true);
            if (_engineHost) {
                _engineHost->shutdown();
            }
            server.stop();
            io.stop();
        }
    );

    // Set up diagnostics timer (emit every 5 seconds)
    asio::steady_timer diagnosticsTimer(io);
    std::function<void()> scheduleDiagnostics;
    scheduleDiagnostics = [&diagnosticsTimer, &server, this, &shuttingDown, &scheduleDiagnostics]() {
        diagnosticsTimer.expires_after(std::chrono::seconds(5));
        diagnosticsTimer.async_wait(
            [&server, this, &shuttingDown, &scheduleDiagnostics](std::error_code ec) {
                if (ec || shuttingDown.load()) {
                    return;
                }

                // Send diagnostics event to all connected clients
                server.broadcastDiagnostics(_engineHost.get());

                // Schedule next diagnostics
                scheduleDiagnostics();
            }
        );
    };
    scheduleDiagnostics();

    // Set up metering timer (emit every 50ms, ~20 Hz for smooth UI updates)
    asio::steady_timer meteringTimer(io);
    std::function<void()> scheduleMetering;
    scheduleMetering = [&meteringTimer, &server, this, &shuttingDown, &scheduleMetering]() {
        meteringTimer.expires_after(std::chrono::milliseconds(50));
        meteringTimer.async_wait(
            [&server, this, &shuttingDown, &scheduleMetering](std::error_code ec) {
                if (ec || shuttingDown.load()) {
                    return;
                }

                // Only send metering when engine is running
                if (_engineHost && _engineHost->state() == EngineHost::State::Running) {
                    server.broadcastMetering(&_engineHost->metering());
                }

                // Schedule next metering update
                scheduleMetering();
            }
        );
    };
    scheduleMetering();

    // Start server
    server.start();

    // Run IO loop
    std::cout << "[SignalApp] Starting IO loop..." << std::endl;
    io.run();

    std::cout << "[SignalApp] IO loop finished" << std::endl;
    return 0;
}

