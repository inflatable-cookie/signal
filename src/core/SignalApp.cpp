#include "core/SignalApp.hpp"
#include "ipc/TcpServer.hpp"
#include "ipc/DomainDispatcher.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "core/EngineHost.hpp"
#include "core/PluginHost.hpp"
#include "core/MeteringService.hpp"
#include "core/MidiInputRouter.hpp"
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
#include <nlohmann/json.hpp>
#include <algorithm>
#include <atomic>
#include <chrono>
#include <cstdlib>
#include <functional>
#include <iostream>
#include <optional>
#include <signal.h>
#include <thread>
#include <tuple>
#include <unordered_set>

namespace {

std::vector<MidiInputDeviceInfo> normaliseMidiDeviceSnapshot(
    std::vector<MidiInputDeviceInfo> devices
) {
    std::sort(
        devices.begin(),
        devices.end(),
        [](const MidiInputDeviceInfo& left, const MidiInputDeviceInfo& right) {
            return left.id < right.id;
        }
    );

    return devices;
}

bool midiDeviceSnapshotsEqual(
    const std::vector<MidiInputDeviceInfo>& left,
    const std::vector<MidiInputDeviceInfo>& right
) {
    if (left.size() != right.size()) {
        return false;
    }

    for (size_t index = 0; index < left.size(); ++index) {
        const auto& left_device = left[index];
        const auto& right_device = right[index];

        if (
            left_device.id != right_device.id
            || left_device.name != right_device.name
            || left_device.manufacturer != right_device.manufacturer
            || left_device.api != right_device.api
            || left_device.container_id != right_device.container_id
            || left_device.device_id != right_device.device_id
            || left_device.port_handle != right_device.port_handle
            || left_device.port_name != right_device.port_name
            || left_device.device_name != right_device.device_name
            || left_device.display_name != right_device.display_name
            || left_device.product != right_device.product
            || left_device.serial != right_device.serial
            || left_device.last_seen_timestamp_ms != right_device.last_seen_timestamp_ms
            || left_device.is_connected != right_device.is_connected
        ) {
            return false;
        }
    }

    return true;
}

nlohmann::json buildControlDeviceInventoryPayload(
    const std::vector<MidiInputDeviceInfo>& devices
) {
    nlohmann::json payload;
    nlohmann::json device_array = nlohmann::json::array();

    for (const auto& device : devices) {
        nlohmann::json device_json;
        device_json["id"] = device.id;
        device_json["kind"] = "midi";
        device_json["name"] = device.name;
        device_json["manufacturer"] = device.manufacturer;
        device_json["connectionState"] = device.is_connected ? "connected" : "disconnected";

        if (!device.api.empty()) {
            device_json["portApi"] = device.api;
        }
        if (!device.container_id.empty()) {
            device_json["portContainerId"] = device.container_id;
        }
        if (!device.device_id.empty()) {
            device_json["portDeviceId"] = device.device_id;
        }
        if (device.port_handle.has_value()) {
            device_json["portHandle"] = device.port_handle.value();
        }
        if (!device.port_name.empty()) {
            device_json["portName"] = device.port_name;
        }
        if (!device.device_name.empty()) {
            device_json["deviceName"] = device.device_name;
        }
        if (!device.display_name.empty()) {
            device_json["displayName"] = device.display_name;
        }
        if (!device.product.empty()) {
            device_json["product"] = device.product;
        }
        if (!device.serial.empty()) {
            device_json["serial"] = device.serial;
        }
        if (device.last_seen_timestamp_ms.has_value()) {
            device_json["lastSeenTimestampMs"] = device.last_seen_timestamp_ms.value();
        }

        device_array.push_back(device_json);
    }

    payload["devices"] = device_array;

    return payload;
}

std::vector<MidiInputDeviceInfo> mergeMidiDeviceSnapshot(
    std::unordered_map<std::string, MidiInputDeviceInfo>& registry,
    const std::vector<MidiInputDeviceInfo>& current,
    std::uint64_t now_ms
) {
    std::unordered_set<std::string> seen;
    seen.reserve(current.size());

    for (const auto& device : current) {
        auto it = registry.find(device.id);
        MidiInputDeviceInfo entry = device;
        bool was_connected = it != registry.end() && it->second.is_connected;

        if (!was_connected) {
            entry.last_seen_timestamp_ms = now_ms;
        } else if (it != registry.end()) {
            entry.last_seen_timestamp_ms = it->second.last_seen_timestamp_ms;
        }

        entry.is_connected = true;
        registry[device.id] = entry;
        seen.insert(device.id);
    }

    for (auto& [id, entry] : registry) {
        if (seen.find(id) == seen.end()) {
            if (entry.is_connected) {
                entry.last_seen_timestamp_ms = now_ms;
            }

            entry.is_connected = false;
        }
    }

    std::vector<MidiInputDeviceInfo> snapshot;
    snapshot.reserve(registry.size());
    for (const auto& [id, entry] : registry) {
        snapshot.push_back(entry);
    }

    return normaliseMidiDeviceSnapshot(std::move(snapshot));
}

} // namespace

SignalApp::SignalApp() {
    // Initialize unified logging system
    initLogging();

    LOG_DEBUG({"SignalApp"}, "Initialising...");

    try {
        _engineHost = std::make_unique<EngineHost>();

        LOG_DEBUG({"SignalApp"}, "Initialised");
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
    LOG_DEBUG({"SignalApp"}, "Running...");

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

    MidiInputRouter midiInputRouter;
    midiInputRouter.setEventCallback(
        [&server, &io](
            const std::string& deviceId,
            const std::string& controlKey,
            const std::string& action,
            std::optional<double> value
        ) {
            nlohmann::json payload;
            payload["deviceId"] = deviceId;
            payload["controlKey"] = controlKey;
            payload["action"] = action;
            payload["timestampMs"] = static_cast<std::uint64_t>(
                std::chrono::duration_cast<std::chrono::milliseconds>(
                    std::chrono::system_clock::now().time_since_epoch()
                ).count()
            );

            if (value.has_value()) {
                payload["value"] = value.value();
            }

            io.post([payload = std::move(payload), &server]() mutable {
                server.broadcastControlEvent(payload);
            });
        }
    );

    // Set up signal handling for graceful shutdown
    asio::signal_set signals(io, SIGINT, SIGTERM);
    std::atomic<bool> shuttingDown{false};

    signals.async_wait(
        [&server, &io, this, &shuttingDown, &midiInputRouter](std::error_code /*ec*/, int /*signo*/) {
            LOG_INFO({"SignalApp"}, "Shutdown signal received");
            shuttingDown.store(true);
            _shutdownRequested.store(true);
            _pluginScanThread.request_stop();

            if (_engineHost) {
                _engineHost->shutdown();
            }

            midiInputRouter.shutdown();
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

    // Set up MIDI device inventory polling
    asio::steady_timer midiInventoryTimer(io);
    std::function<void()> scheduleMidiInventoryPoll;
    constexpr std::chrono::seconds MIDI_INVENTORY_POLL_INTERVAL{2};

    auto captureMidiDeviceSnapshot = [this]() -> std::vector<MidiInputDeviceInfo> {
        if (!_engineHost) {
            return {};
        }

        auto now_ms = static_cast<std::uint64_t>(
            std::chrono::duration_cast<std::chrono::milliseconds>(
                std::chrono::system_clock::now().time_since_epoch()
            ).count()
        );
        auto devices = _engineHost->enumerateMidiInputDevices();
        return mergeMidiDeviceSnapshot(_midiDeviceRegistry, devices, now_ms);
    };

    scheduleMidiInventoryPoll = [
        &midiInventoryTimer,
        &server,
        &midiInputRouter,
        this,
        &shuttingDown,
        &scheduleMidiInventoryPoll,
        captureMidiDeviceSnapshot,
        MIDI_INVENTORY_POLL_INTERVAL
    ]() mutable {
        midiInventoryTimer.expires_after(MIDI_INVENTORY_POLL_INTERVAL);
        midiInventoryTimer.async_wait(
            [
                &server,
                this,
                &midiInputRouter,
                &shuttingDown,
                &scheduleMidiInventoryPoll,
                captureMidiDeviceSnapshot
            ](std::error_code ec) mutable {
                if (ec || shuttingDown.load()) {
                    return;
                }

                auto snapshot = captureMidiDeviceSnapshot();

                midiInputRouter.refreshInputs();

                if (!midiDeviceSnapshotsEqual(snapshot, _lastMidiDeviceSnapshot)) {
                    _lastMidiDeviceSnapshot = snapshot;
                    auto payload = buildControlDeviceInventoryPayload(_lastMidiDeviceSnapshot);
                    server.broadcastControlDeviceInventory(payload);
                }

                scheduleMidiInventoryPoll();
            }
        );
    };
    scheduleMidiInventoryPoll();

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

    scheduleOrphanCheck = [&server, &orphanShutdownTimer, &io, this, &shuttingDown, &scheduleOrphanCheck, ORPHAN_GRACE_PERIOD_MS, &midiInputRouter]() {
        // Only schedule check if we've seen a client and have no active clients
        if (server.hasEverSeenClient() && server.getActiveClientCount() == 0) {
            LOG_INFO({"SignalApp", "Lifecycle", "Orphan"}, "Last client disconnected; scheduling orphan check in 5000ms");
            orphanShutdownTimer.expires_after(ORPHAN_GRACE_PERIOD_MS);
            orphanShutdownTimer.async_wait(
                [&server, &io, this, &shuttingDown, &midiInputRouter](std::error_code ec) {
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

                        midiInputRouter.shutdown();
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
        [&server, &orphanShutdownTimer, this, captureMidiDeviceSnapshot, &midiInputRouter](const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session) {
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

            midiInputRouter.refreshInputs();

            auto snapshot = captureMidiDeviceSnapshot();
            _lastMidiDeviceSnapshot = snapshot;
            auto payload = buildControlDeviceInventoryPayload(_lastMidiDeviceSnapshot);
            server.sendControlDeviceInventory(payload, session);
        }
    );

    // Start server
    server.start();

    // Signal startup stays idle by default.
    // Pulse is the lifecycle orchestrator and must explicitly request:
    // - engine start/stop via engine domain commands
    // - plugin scanning via plugin domain commands

    // Run IO loop
    LOG_DEBUG({"SignalApp"}, "Starting IO loop...");
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
