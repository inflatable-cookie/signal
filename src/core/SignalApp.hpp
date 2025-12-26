#pragma once

/// SignalApp - Main application entry point
///
/// Thread: Main thread
/// Ownership: Owned by main()
/// Communication:
///   - Owns EngineHost
///   - Sets up IPC server with Asio io_context
///   - Coordinates shutdown

#include <memory>
#include <atomic>
#include <thread>

class EngineHost;

class SignalApp {
public:
    SignalApp();
    ~SignalApp();

    int run();

    // Request shutdown due to orphaned state (no active clients after having seen at least one)
    void requestShutdownDueToOrphanedState();

private:
    std::unique_ptr<EngineHost> _engineHost;
    std::atomic<bool> _shutdownRequested{false};
    std::jthread _pluginScanThread;
};
