#pragma once

#include <memory>

class IpcRouter;
class EngineHost;

class SignalApp {
public:
    SignalApp();
    ~SignalApp();

    int run();

private:
    std::unique_ptr<IpcRouter> _router;
    std::unique_ptr<EngineHost> _engineHost;
};

