#pragma once

/// MixerDomain - IPC domain handler for mixer updates
///
/// Thread: IPC thread (Asio handler context)
/// Ownership: Owned by IpcRouter
/// Communication:
///   - Receives commands from Pulse (mixer.updateChannel)
///   - Updates MixerService state
///   - Changes are applied in real-time by audio thread

#include "ipc/Envelope.hpp"
#include <memory>

class EngineHost;

class MixerDomain : public DomainHandler {
public:
    explicit MixerDomain(EngineHost* engineHost);
    ~MixerDomain() override = default;

    void handle(const Envelope& env) override;

private:
    EngineHost* _engineHost;
};

