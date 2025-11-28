#include "core/SignalApp.hpp"
#include "ipc/Router.hpp"
#include "ipc/TcpServer.hpp"
#include "ipc/DomainDispatcher.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "core/EngineHost.hpp"
#include "core/PluginHost.hpp"
#include "core/MeteringService.hpp"
#include "domains/EngineDomain.hpp"
#include "domains/TransportDomain.hpp"
#include "domains/MeteringDomain.hpp"
#include "domains/MixerDomain.hpp"
#include "domains/AutomationDomain.hpp"
#include "domains/AssetsDomain.hpp"
#include "domains/HardwareDomain.hpp"
#include "logging/Logging.hpp"
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
    // Initialize unified logging system
    initLogging();

    LOG_INFO({"SignalApp"}, "Initialising...");

    try {
        _engineHost = std::make_unique<EngineHost>();
        _router = std::make_unique<IpcRouter>();

        // Register domain handlers
        auto engineDomain = std::make_shared<EngineDomain>(_engineHost.get());
        _router->registerHandler("engine", engineDomain);

        auto transportDomain = std::make_shared<TransportDomain>(_engineHost.get());
        _router->registerHandler("transport", transportDomain);

        auto meteringDomain = std::make_shared<MeteringDomain>(&_engineHost->metering(), _engineHost.get());
        _router->registerHandler("metering", meteringDomain);

        auto mixerDomain = std::make_shared<MixerDomain>(_engineHost.get());
        _router->registerHandler("mixer", mixerDomain);

        auto automationDomain = std::make_shared<AutomationDomain>(_engineHost.get());
        _router->registerHandler("automation", automationDomain);

        auto assetsDomain = std::make_shared<AssetsDomain>(_engineHost.get());
        _router->registerHandler("assets", assetsDomain);

        // Register hardware domain
        auto hardwareDomain = std::make_shared<HardwareDomain>(_engineHost.get());
        _router->registerHandler("hardware", hardwareDomain);

        LOG_INFO({"SignalApp"}, "Initialised");
    } catch (const std::exception& e) {
        LOG_ERROR({"SignalApp"}, std::string("Error during initialization: ") + e.what());
        throw;
    } catch (...) {
        LOG_ERROR({"SignalApp"}, "Unknown error during initialization");
        throw;
    }
}

SignalApp::~SignalApp() {
    LOG_INFO({"SignalApp"}, "Shutting down...");
}

int SignalApp::run() {
    LOG_INFO({"SignalApp"}, "Running...");

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
            LOG_WARN({"SignalApp"}, "Invalid SIGNAL_PORT, using default 7888");
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
            LOG_INFO({"SignalApp"}, "Shutdown signal received");
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

    // Set up transport position update timer (emit every 50ms, ~20 Hz for smooth playhead updates)
    asio::steady_timer transportPositionTimer(io);
    std::function<void()> scheduleTransportPosition;
    scheduleTransportPosition = [&transportPositionTimer, &server, this, &shuttingDown, &scheduleTransportPosition]() {
        transportPositionTimer.expires_after(std::chrono::milliseconds(50));
        transportPositionTimer.async_wait(
            [&server, this, &shuttingDown, &scheduleTransportPosition](std::error_code ec) {
                if (ec || shuttingDown.load()) {
                    return;
                }

                // Only send position updates when engine is running
                if (_engineHost && _engineHost->state() == EngineHost::State::Running) {
                    server.broadcastTransportPosition(_engineHost.get());
                }

                // Schedule next position update
                scheduleTransportPosition();
            }
        );
    };
    scheduleTransportPosition();

    // Set up callback to send initial engine state when a client connects
    server.setClientConnectedCallback(
        [&server, this](const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session) {
            if (_engineHost) {
                LOG_INFO({"SignalApp"}, "Client connected, sending initial engine state...");
                server.sendEngineState(_engineHost.get(), session);
            }
        }
    );

    // Start server
    server.start();

    // Auto-start the audio engine when Signal boots up
    if (_engineHost) {
        LOG_INFO({"SignalApp"}, "Auto-starting audio engine...");
        _engineHost->start();
        // Send initial engine state to any connected clients
        // (This will be sent automatically when clients connect via the callback,
        // but we also send it now in case a client is already connected)
        server.broadcastEngineState(_engineHost.get());
    }

    // Scan for plugins after server starts (deferred to prevent blocking startup)
    // This allows Signal to accept connections even if plugin scanning fails
    if (_engineHost && _engineHost->pluginHost()) {
        LOG_INFO({"SignalApp"}, "Scanning for CLAP plugins...");
        _engineHost->pluginHost()->scanPlugins();
    }

    // Run IO loop
    LOG_INFO({"SignalApp"}, "Starting IO loop...");
    io.run();

    LOG_INFO({"SignalApp"}, "IO loop finished");
    return 0;
}

