#pragma once

#include "ipc/IpcDomainHandler.hpp"

class EngineHost;

class RecordingDomain : public loophole::signal::ipc::IpcDomainHandler {
public:
    explicit RecordingDomain(EngineHost* engineHost);
    ~RecordingDomain() override = default;

    void handle(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    ) override;

private:
    EngineHost* _engineHost;

    void handleSetArmState(const nlohmann::json& payload);
    void handleStartRecording(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );
    void handleStopRecording(
        const loophole::signal::ipc::IpcEnvelope& env,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session
    );
    void emitRecordingStateEvent(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
        bool isRecording,
        const std::string& recordId,
        std::optional<std::uint64_t> endSample = std::nullopt
    );
    void emitRecordingFinishedEvent(
        const loophole::signal::ipc::IpcEnvelope& commandEnv,
        const std::shared_ptr<loophole::signal::ipc::TcpClientSession>& session,
        const std::string& recordId,
        std::uint64_t endSample
    );
};
