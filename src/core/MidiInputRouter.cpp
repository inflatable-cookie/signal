#include "core/MidiInputRouter.hpp"
#include "backend/MidiDeviceIdentity.hpp"
#include "domains/control/MidiControlNormaliser.hpp"
#include "logging/Logging.hpp"
#include <libremidi/libremidi.hpp>
#include <cstdint>
#include <unordered_map>

namespace {

struct MidiInputConnection {
    std::string device_id;
    libremidi::input_port port;
    std::unique_ptr<libremidi::midi_in> input;
};

struct MidiInputRouterImpl {
    libremidi::observer observer;
    std::unordered_map<std::string, MidiInputConnection> inputs;
    MidiInputRouter::ControlEventCallback callback;

    void refreshInputs() {
        std::vector<libremidi::input_port> ports;

        try {
            ports = observer.get_input_ports();
        } catch (const std::exception& e) {
            LOG_WARN({"MidiInputRouter"}, std::string("Failed to enumerate MIDI inputs: ") + e.what());
            return;
        }

        std::unordered_map<std::string, libremidi::input_port> ports_by_id;
        ports_by_id.reserve(ports.size());

        for (const auto& port : ports) {
            auto device_id = loophole::signal::midi::buildStableMidiDeviceId(port);
            ports_by_id.emplace(device_id, port);
        }

        for (auto it = inputs.begin(); it != inputs.end();) {
            if (ports_by_id.find(it->first) == ports_by_id.end()) {
                if (it->second.input) {
                    it->second.input->close_port();
                }

                it = inputs.erase(it);
                continue;
            }

            ++it;
        }

        for (const auto& [device_id, port] : ports_by_id) {
            if (inputs.find(device_id) != inputs.end()) {
                continue;
            }

            libremidi::input_configuration config;
            config.ignore_sysex = true;
            config.ignore_timing = false;
            config.ignore_sensing = true;
            config.on_message = [device_id, cb = callback](libremidi::message&& message) {
                if (!cb) {
                    return;
                }

                if (message.bytes.empty()) {
                    return;
                }

                std::uint8_t status = message.bytes[0];
                std::uint8_t data1 = message.bytes.size() > 1 ? message.bytes[1] : 0;
                std::uint8_t data2 = message.bytes.size() > 2 ? message.bytes[2] : 0;

                auto normalised = loophole::signal::control::normaliseMidiMessage(status, data1, data2);
                if (!normalised.has_value()) {
                    return;
                }

                cb(device_id, normalised->control_key, normalised->action, normalised->value);
            };

            auto input = std::make_unique<libremidi::midi_in>(
                config,
                libremidi::midi_in_configuration_for(observer)
            );

            if (auto err = input->open_port(port); err != stdx::error{}) {
                LOG_WARN({"MidiInputRouter"}, "Failed to open MIDI input port");
                continue;
            }

            inputs.emplace(device_id, MidiInputConnection{
                device_id,
                port,
                std::move(input)
            });
        }
    }

    void shutdown() {
        for (auto& [id, connection] : inputs) {
            if (connection.input) {
                connection.input->close_port();
            }
        }

        inputs.clear();
    }
};

} // namespace

MidiInputRouter::MidiInputRouter()
    : _impl(std::make_unique<MidiInputRouterImpl>())
{
}

MidiInputRouter::~MidiInputRouter() {
    shutdown();
}

void MidiInputRouter::setEventCallback(ControlEventCallback callback) {
    _impl->callback = std::move(callback);
}

void MidiInputRouter::refreshInputs() {
    _impl->refreshInputs();
}

void MidiInputRouter::shutdown() {
    if (_impl) {
        _impl->shutdown();
    }
}
