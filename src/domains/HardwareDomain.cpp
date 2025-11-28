#include "domains/HardwareDomain.hpp"
#include "core/EngineHost.hpp"
#include "backend/OutputDeviceInfo.hpp"
#include "ipc/Envelope.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <sstream>

HardwareDomain::HardwareDomain(EngineHost* engineHost) : _engineHost(engineHost) {
}

void HardwareDomain::handle(const Envelope& env) {
    if (env.kind != "command") {
        LOG_DEBUG({"HardwareDomain"}, std::string("Ignoring non-command: ") + env.kind);
        return;
    }

    if (!_engineHost) {
        LOG_ERROR({"HardwareDomain"}, "EngineHost is null");
        return;
    }

    // HardwareDomain processes commands via router
    // Event emission is handled by DomainDispatcher which calls handleListOutputDevices
    // or handleSelectOutputDevice directly to get response data
    if (env.name == "listOutputDevices" || env.name == "refreshOutputDevices") {
        LOG_DEBUG({"HardwareDomain"}, "Device list command received (processing via router)");
    } else if (env.name == "selectOutputDevice" || env.name == "setActiveOutputDevice") {
        // Parse device ID from payload
        std::string deviceId;
        try {
            nlohmann::json payload = nlohmann::json::parse(env.payload);
            if (payload.contains("id") && payload["id"].is_string()) {
                deviceId = payload["id"].get<std::string>();
            } else if (payload.contains("deviceId") && payload["deviceId"].is_string()) {
                deviceId = payload["deviceId"].get<std::string>();
            } else {
                LOG_ERROR({"HardwareDomain"}, "selectOutputDevice: missing or invalid 'id' or 'deviceId' field");
                return;
            }
        } catch (const std::exception& e) {
            LOG_ERROR({"HardwareDomain"}, std::string("Failed to parse selectOutputDevice payload: ") + e.what());
            return;
        }

        // Attempt to set the device
        bool success = _engineHost->setOutputDevice(deviceId);
        if (success) {
            std::ostringstream msg;
            msg << "Device selection succeeded: " << deviceId;
            LOG_INFO({"HardwareDomain"}, msg.str());
        } else {
            std::ostringstream msg;
            msg << "Device selection failed: " << deviceId;
            LOG_WARN({"HardwareDomain"}, msg.str());
        }
    } else {
        LOG_WARN({"HardwareDomain"}, std::string("Unknown command: ") + env.name);
    }
}

std::optional<HardwareResponse> HardwareDomain::handleListOutputDevices() {
    if (!_engineHost) {
        return std::nullopt;
    }

    // Enumerate output devices
    auto devices = _engineHost->enumerateOutputDevices();
    std::string activeDeviceId = _engineHost->getActiveOutputDeviceId();

    nlohmann::json payload;
    nlohmann::json devicesArray = nlohmann::json::array();

    for (const auto& device : devices) {
        nlohmann::json deviceJson;
        deviceJson["id"] = device.id;
        deviceJson["name"] = device.name;
        deviceJson["isDefault"] = device.isDefault;
        deviceJson["isActive"] = (device.id == activeDeviceId);
        deviceJson["maxChannels"] = device.maxChannels;
        deviceJson["preferredSampleRate"] = device.preferredSampleRate;
        devicesArray.push_back(deviceJson);
    }

    payload["devices"] = devicesArray;
    payload["activeDeviceId"] = activeDeviceId;

    HardwareResponse response;
    response.eventName = "outputDevicesListed";
    response.payload = payload;

    std::ostringstream msg;
    msg << "Listed " << devices.size() << " output devices";
    LOG_INFO({"HardwareDomain"}, msg.str());

    return response;
}

std::optional<HardwareResponse> HardwareDomain::handleSelectOutputDevice(const std::string& deviceId) {
    if (!_engineHost) {
        return std::nullopt;
    }

    // Attempt to set the device
    bool success = _engineHost->setOutputDevice(deviceId);

    nlohmann::json payload;
    payload["success"] = success;
    payload["deviceId"] = deviceId;

    if (success) {
        // Refresh device list to get updated active status
        auto devices = _engineHost->enumerateOutputDevices();
        std::string activeDeviceId = _engineHost->getActiveOutputDeviceId();

        nlohmann::json devicesArray = nlohmann::json::array();
        for (const auto& device : devices) {
            nlohmann::json deviceJson;
            deviceJson["id"] = device.id;
            deviceJson["name"] = device.name;
            deviceJson["isDefault"] = device.isDefault;
            deviceJson["isActive"] = (device.id == activeDeviceId);
            deviceJson["maxChannels"] = device.maxChannels;
            deviceJson["preferredSampleRate"] = device.preferredSampleRate;
            devicesArray.push_back(deviceJson);
        }
        payload["devices"] = devicesArray;
        payload["activeDeviceId"] = activeDeviceId;

        std::ostringstream msg;
        msg << "Device selection succeeded: " << deviceId;
        LOG_INFO({"HardwareDomain"}, msg.str());
    } else {
        payload["error"] = "Failed to switch to device: " + deviceId;
        std::ostringstream msg;
        msg << "Device selection failed: " << deviceId;
        LOG_WARN({"HardwareDomain"}, msg.str());
    }

    HardwareResponse response;
    response.eventName = "outputDeviceSelected";
    response.payload = payload;

    return response;
}

