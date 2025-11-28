#include "domains/HardwareDomain.hpp"
#include "core/EngineHost.hpp"
#include "backend/OutputDeviceInfo.hpp"
#include "ipc/IpcEnvelope.hpp"
#include "ipc/TcpClientSession.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <sstream>

HardwareDomain::HardwareDomain(EngineHost* engineHost)
    : _engineHost(engineHost)
{
}

void HardwareDomain::handle(
    const loophole::signal::ipc::IpcEnvelope& env,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    if (env.domain != "hardware") {
        LOG_DEBUG({"HardwareDomain"}, "Received envelope for different domain");
        return;
    }

    if (env.kind != loophole::signal::ipc::IpcKind::Command) {
        LOG_DEBUG({"HardwareDomain"}, "Ignoring non-command hardware envelope");
        return;
    }

    if (!_engineHost) {
        LOG_ERROR({"HardwareDomain"}, "EngineHost is null");
        return;
    }

    // Handle device selection commands directly
    if (env.name == "selectOutputDevice" || env.name == "setActiveOutputDevice") {
        // Parse device ID from payload and set device
        std::string deviceId;
        if (env.payload.contains("id") && env.payload["id"].is_string()) {
            deviceId = env.payload["id"].get<std::string>();
        } else if (env.payload.contains("deviceId") && env.payload["deviceId"].is_string()) {
            deviceId = env.payload["deviceId"].get<std::string>();
        } else {
            LOG_ERROR({"HardwareDomain"}, "selectOutputDevice: missing or invalid 'id' or 'deviceId' field");
            // Send error response
            using namespace loophole::signal::ipc;
            IpcEnvelope errorResponse;
            errorResponse.version = 1;
            errorResponse.id = "hardware-outputDeviceSelected-" + env.id;
            errorResponse.correlationId = env.id;
            errorResponse.timestamp = currentTimestamp();
            errorResponse.origin = IpcOrigin::Signal;
            switch (env.origin) {
            case IpcOrigin::Aura: errorResponse.target = IpcTarget::Aura; break;
            case IpcOrigin::Pulse: errorResponse.target = IpcTarget::Pulse; break;
            case IpcOrigin::Signal: errorResponse.target = IpcTarget::Signal; break;
            case IpcOrigin::Composer: errorResponse.target = IpcTarget::Composer; break;
            }
            errorResponse.domain = "hardware";
            errorResponse.kind = IpcKind::Event;
            errorResponse.name = "outputDeviceSelected";
            errorResponse.priority = env.priority;
            nlohmann::json errorPayload;
            errorPayload["success"] = false;
            errorPayload["error"] = "Missing or invalid device ID";
            errorResponse.payload = errorPayload;
            session->send(errorResponse);
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
    }

    // Handle responses based on command name
    if (env.name == "listOutputDevices" || env.name == "refreshOutputDevices") {
        sendListOutputDevicesResponse(env, session);
    } else if (env.name == "selectOutputDevice" || env.name == "setActiveOutputDevice") {
        // Parse device ID from payload
        std::string deviceId;
        try {
            if (env.payload.contains("id") && env.payload["id"].is_string()) {
                deviceId = env.payload["id"].get<std::string>();
            } else if (env.payload.contains("deviceId") && env.payload["deviceId"].is_string()) {
                deviceId = env.payload["deviceId"].get<std::string>();
            } else {
                LOG_ERROR({"HardwareDomain"}, "selectOutputDevice: missing or invalid 'id' or 'deviceId' field");
                // Send error response
                using namespace loophole::signal::ipc;
                IpcEnvelope errorResponse;
                errorResponse.version = 1;
                errorResponse.id = "hardware-outputDeviceSelected-" + env.id;
                errorResponse.correlationId = env.id;
                errorResponse.timestamp = currentTimestamp();
                errorResponse.origin = IpcOrigin::Signal;
                switch (env.origin) {
                case IpcOrigin::Aura: errorResponse.target = IpcTarget::Aura; break;
                case IpcOrigin::Pulse: errorResponse.target = IpcTarget::Pulse; break;
                case IpcOrigin::Signal: errorResponse.target = IpcTarget::Signal; break;
                case IpcOrigin::Composer: errorResponse.target = IpcTarget::Composer; break;
                }
                errorResponse.domain = "hardware";
                errorResponse.kind = IpcKind::Event;
                errorResponse.name = "outputDeviceSelected";
                errorResponse.priority = env.priority;
                nlohmann::json errorPayload;
                errorPayload["success"] = false;
                errorPayload["error"] = "Missing or invalid device ID";
                errorResponse.payload = errorPayload;
                session->send(errorResponse);
                return;
            }
        } catch (const std::exception& e) {
            LOG_ERROR({"HardwareDomain"}, std::string("Failed to parse selectOutputDevice payload: ") + e.what());
            // Send error response
            using namespace loophole::signal::ipc;
            IpcEnvelope errorResponse;
            errorResponse.version = 1;
            errorResponse.id = "hardware-outputDeviceSelected-" + env.id;
            errorResponse.correlationId = env.id;
            errorResponse.timestamp = currentTimestamp();
            errorResponse.origin = IpcOrigin::Signal;
            switch (env.origin) {
            case IpcOrigin::Aura: errorResponse.target = IpcTarget::Aura; break;
            case IpcOrigin::Pulse: errorResponse.target = IpcTarget::Pulse; break;
            case IpcOrigin::Signal: errorResponse.target = IpcTarget::Signal; break;
            case IpcOrigin::Composer: errorResponse.target = IpcTarget::Composer; break;
            }
            errorResponse.domain = "hardware";
            errorResponse.kind = IpcKind::Event;
            errorResponse.name = "outputDeviceSelected";
            errorResponse.priority = env.priority;
            nlohmann::json errorPayload;
            errorPayload["success"] = false;
            errorPayload["error"] = "Failed to parse payload: " + std::string(e.what());
            errorResponse.payload = errorPayload;
            session->send(errorResponse);
            return;
        }

        sendSelectOutputDeviceResponse(env, session, deviceId);
    } else {
        LOG_WARN({"HardwareDomain"}, std::string("Unknown hardware command: ") + env.name);
    }
}

void HardwareDomain::sendListOutputDevicesResponse(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
) {
    using namespace loophole::signal::ipc;

    if (!_engineHost) {
        return;
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

    IpcEnvelope response;
    response.version = 1;
    response.id = "hardware-outputDevicesListed-" + commandEnv.id;
    response.correlationId = commandEnv.id;
    response.timestamp = currentTimestamp();
    response.origin = IpcOrigin::Signal;

    switch (commandEnv.origin) {
    case IpcOrigin::Aura:
        response.target = IpcTarget::Aura;
        break;
    case IpcOrigin::Pulse:
        response.target = IpcTarget::Pulse;
        break;
    case IpcOrigin::Signal:
        response.target = IpcTarget::Signal;
        break;
    case IpcOrigin::Composer:
        response.target = IpcTarget::Composer;
        break;
    }

    response.domain = "hardware";
    response.kind = IpcKind::Event;
    response.name = "outputDevicesListed";
    response.priority = commandEnv.priority;
    response.payload = payload;

    session->send(response);

    std::ostringstream msg;
    msg << "Listed " << devices.size() << " output devices";
    LOG_INFO({"HardwareDomain"}, msg.str());
}

void HardwareDomain::sendSelectOutputDeviceResponse(
    const loophole::signal::ipc::IpcEnvelope& commandEnv,
    const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
    const std::string& deviceId
) {
    using namespace loophole::signal::ipc;

    if (!_engineHost) {
        return;
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

    IpcEnvelope response;
    response.version = 1;
    response.id = "hardware-outputDeviceSelected-" + commandEnv.id;
    response.correlationId = commandEnv.id;
    response.timestamp = currentTimestamp();
    response.origin = IpcOrigin::Signal;

    switch (commandEnv.origin) {
    case IpcOrigin::Aura:
        response.target = IpcTarget::Aura;
        break;
    case IpcOrigin::Pulse:
        response.target = IpcTarget::Pulse;
        break;
    case IpcOrigin::Signal:
        response.target = IpcTarget::Signal;
        break;
    case IpcOrigin::Composer:
        response.target = IpcTarget::Composer;
        break;
    }

    response.domain = "hardware";
    response.kind = IpcKind::Event;
    response.name = "outputDeviceSelected";
    response.priority = commandEnv.priority;
    response.payload = payload;

    session->send(response);
}

