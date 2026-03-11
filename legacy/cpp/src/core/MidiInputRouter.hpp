#pragma once

#include <functional>
#include <memory>
#include <optional>
#include <string>

class MidiInputRouter {
public:
    using ControlEventCallback = std::function<void(
        const std::string& deviceId,
        const std::string& controlKey,
        const std::string& action,
        std::optional<double> value
    )>;

    MidiInputRouter();
    ~MidiInputRouter();

    MidiInputRouter(const MidiInputRouter&) = delete;
    MidiInputRouter& operator=(const MidiInputRouter&) = delete;
    MidiInputRouter(MidiInputRouter&&) noexcept = delete;
    MidiInputRouter& operator=(MidiInputRouter&&) noexcept = delete;

    void setEventCallback(ControlEventCallback callback);
    void refreshInputs();
    void shutdown();

private:
    struct Impl;
    std::unique_ptr<Impl> _impl;
};
