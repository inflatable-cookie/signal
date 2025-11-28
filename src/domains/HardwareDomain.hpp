#pragma once

#include "ipc/Router.hpp"
#include <memory>
#include <optional>
#include <string>
#include <nlohmann/json.hpp>

class EngineHost;

// Response data structure for hardware commands
struct HardwareResponse {
    std::string eventName;
    nlohmann::json payload;
};

class HardwareDomain : public DomainHandler {
public:
    explicit HardwareDomain(EngineHost* engineHost);
    ~HardwareDomain() override = default;

    // DomainHandler interface - processes commands via router
    void handle(const Envelope& env) override;

    // Direct methods for DomainDispatcher to get response data
    // These are called directly (not through router) when events need to be sent
    std::optional<HardwareResponse> handleListOutputDevices();
    std::optional<HardwareResponse> handleSelectOutputDevice(const std::string& deviceId);

private:
    EngineHost* _engineHost;
};

