#include "core/SignalApp.hpp"
#include "ipc/Router.hpp"
#include "core/EngineHost.hpp"
#include "domains/EngineDomain.hpp"
#include "domains/TransportDomain.hpp"
#include <iostream>

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
    std::cout << "[SignalApp] Running (stub mode)" << std::endl;
    // For now, just initialise and return success
    // Later: start IPC server and event loop
    return 0;
}

