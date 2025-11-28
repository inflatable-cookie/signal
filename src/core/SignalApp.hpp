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

class EngineHost;

class SignalApp {
public:
    SignalApp();
    ~SignalApp();

    int run();

private:
    std::unique_ptr<EngineHost> _engineHost;
};

