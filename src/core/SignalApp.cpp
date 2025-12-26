#include "core/SignalApp.hpp"
#include "ipc/TcpServer.hpp"
#include "ipc/DomainDispatcher.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "core/EngineHost.hpp"
#include "core/PluginHost.hpp"
#include "core/MeteringService.hpp"
#include "domains/EngineDomain.hpp"
#include "domains/TransportDomain.hpp"
#include "domains/MeteringDomain.hpp"
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
    loophole::signal::ipc::DomainDispatcher dispatcher(_engineHost.get(), &_engineHost->metering());

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
            _shutdownRequested.store(true);
            _pluginScanThread.request_stop();
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

    // Set up transport position update timer (emit every ~1s while playing, throttled internally)
    // Immediate updates are triggered by transport state changes (play/stop/seek)
    bool lastTransportPlayingState = false;
    uint64_t lastTransportPositionSamples = 0;
    asio::steady_timer transportPositionTimer(io);
    std::function<void()> scheduleTransportPosition;
    scheduleTransportPosition = [&transportPositionTimer, &server, this, &shuttingDown, &scheduleTransportPosition, &lastTransportPlayingState, &lastTransportPositionSamples]() mutable {
        // Check every 500ms - this is a balance between responsiveness and efficiency
        // The throttling inside broadcastTransportPosition will ensure we only send ~1Hz
        transportPositionTimer.expires_after(std::chrono::milliseconds(500));
        transportPositionTimer.async_wait(
            [&server, this, &shuttingDown, &scheduleTransportPosition, &lastTransportPlayingState, &lastTransportPositionSamples](std::error_code ec) mutable {
                if (ec || shuttingDown.load()) {
                    return;
                }

                // Only send position updates when engine is running
                if (_engineHost && _engineHost->state() == EngineHost::State::Running) {
                    const auto& transport = _engineHost->transport();
                    uint64_t currentPositionSamples = _engineHost->getPlayheadSamples();
                    bool currentPlayingState = transport.isPlaying;

                    // Detect transport state changes (play/stop) or significant position jumps (seek)
                    // Position jumps should only trigger immediate updates for large jumps (seeks),
                    // not normal playback progress. A large jump is > 1 second of samples at 44.1kHz.
                    bool stateChanged = (currentPlayingState != lastTransportPlayingState);
                    int64_t positionDelta = static_cast<int64_t>(currentPositionSamples) - static_cast<int64_t>(lastTransportPositionSamples);
                    bool positionJumped = (lastTransportPositionSamples > 0 && // Only check if we have a previous position
                                         std::abs(positionDelta) > 44100); // More than 1 second at 44.1kHz (indicates a seek, not normal playback)

                    // Force immediate update on state changes or significant position jumps (seek)
                    bool forceImmediate = stateChanged || positionJumped;

                    // Only call broadcastTransportPosition if:
                    // - Force immediate (state change or seek), OR
                    // - Playing (throttling inside will decide if enough time has passed)
                    // Don't call when stopped unless forceImmediate
                    if (forceImmediate || currentPlayingState) {
                        server.broadcastTransportPosition(_engineHost.get(), forceImmediate);
                    }

                    // Update tracking state AFTER calling broadcast (so we detect changes correctly)
                    lastTransportPlayingState = currentPlayingState;
                    lastTransportPositionSamples = currentPositionSamples;
                }

                // Schedule next position update check
                scheduleTransportPosition();
            }
        );
    };
    scheduleTransportPosition();


    // Set up orphan detection with grace period timer
    asio::steady_timer orphanShutdownTimer(io);
    std::function<void()> scheduleOrphanCheck;
    constexpr std::chrono::milliseconds ORPHAN_GRACE_PERIOD_MS{5000}; // 5 seconds

    scheduleOrphanCheck = [&server, &orphanShutdownTimer, &io, this, &shuttingDown, &scheduleOrphanCheck, ORPHAN_GRACE_PERIOD_MS]() {
        // Only schedule check if we've seen a client and have no active clients
        if (server.hasEverSeenClient() && server.getActiveClientCount() == 0) {
            LOG_INFO({"SignalApp", "Lifecycle", "Orphan"}, "Last client disconnected; scheduling orphan check in 5000ms");
            orphanShutdownTimer.expires_after(ORPHAN_GRACE_PERIOD_MS);
            orphanShutdownTimer.async_wait(
                [&server, &io, this, &shuttingDown](std::error_code ec) {
                    if (ec || shuttingDown.load()) {
                        // Timer was cancelled or shutdown already in progress
                        return;
                    }

                    // Check again if we still have no clients
                    if (server.hasEverSeenClient() && server.getActiveClientCount() == 0) {
                        LOG_INFO({"SignalApp", "Lifecycle", "Orphan"}, "No clients after grace period; shutting down Signal");
                        requestShutdownDueToOrphanedState();
                        if (_engineHost) {
                            _engineHost->shutdown();
                        }
                        server.stop();
                        io.stop();
                    }
                }
            );
        }
    };

    // Set up callback for when a client disconnects
    server.setClientDisconnectedCallback([&scheduleOrphanCheck]() {
        scheduleOrphanCheck();
    });

    // Set up callback to cancel orphan shutdown when a client connects
    server.setClientConnectedCallback(
        [&server, &orphanShutdownTimer, this](const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session) {
            // Cancel any pending orphan shutdown timer
            orphanShutdownTimer.cancel();

            // If we had scheduled an orphan check, log that it's cancelled
            if (server.hasEverSeenClient() && server.getActiveClientCount() > 0) {
                LOG_INFO({"SignalApp", "Lifecycle", "Orphan"}, "New client connected; cancelling orphan shutdown");
            }

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

    // Scan for plugins after server starts, but never block IO:
    // run scanning on a background thread so the Asio loop can accept connections.
    if (_engineHost && _engineHost->pluginHost()) {
        LOG_INFO({"SignalApp"}, "Scheduling CLAP plugin scan (background)...");
        _pluginScanThread = std::jthread([this](std::stop_token stopToken) {
            if (!_engineHost || !_engineHost->pluginHost()) {
                return;
            }

            if (stopToken.stop_requested()) {
                return;
            }

            _engineHost->pluginHost()->scanPlugins(stopToken);
        });
    }

    // Run IO loop
    LOG_INFO({"SignalApp"}, "Starting IO loop...");
    io.run();

    LOG_INFO({"SignalApp"}, "IO loop finished");
    return 0;
}

void SignalApp::requestShutdownDueToOrphanedState() {
    if (_shutdownRequested.load()) {
        // Shutdown already in progress
        return;
    }

    _shutdownRequested.store(true);
    _pluginScanThread.request_stop();
    LOG_INFO({"SignalApp", "Lifecycle", "Orphan"}, "Requesting shutdown due to orphaned state");
}
