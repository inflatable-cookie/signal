#include "core/RecordingCapture.hpp"
#include "core/AudioBus.hpp"
#include "logging/Logging.hpp"
#include <algorithm>
#include <sstream>

RecordingSession::RecordingSession()
    : _isRecording(false)
    , _recordingStartSample(0)
    , _audioChunkQueue(CAPTURE_QUEUE_SIZE)
    , _midiChunkQueue(CAPTURE_QUEUE_SIZE)
{
    LOG_DEBUG({"RecordingSession"}, "Created");
}

RecordingSession::~RecordingSession() {
    stopRecording();
    LOG_DEBUG({"RecordingSession"}, "Destroyed");
}

void RecordingSession::startRecording() {
    if (_isRecording.load(std::memory_order_acquire)) {
        LOG_DEBUG({"RecordingSession"}, "Already recording");
        return;
    }

    _recordingStartSample.store(0, std::memory_order_release); // Will be set by EngineHost
    _isRecording.store(true, std::memory_order_release);
    LOG_INFO({"RecordingSession"}, "Recording started");
}

void RecordingSession::stopRecording() {
    if (!_isRecording.load(std::memory_order_acquire)) {
        return;
    }

    _isRecording.store(false, std::memory_order_release);
    LOG_INFO({"RecordingSession"}, "Recording stopped");
}

void RecordingSession::setArmState(const std::string& laneId, bool armed) {
    std::unique_lock<std::shared_mutex> lock(_armStateMutex);
    _armedLanes[laneId].store(armed, std::memory_order_release);
    std::ostringstream msg;
    msg << "Lane " << laneId << " arm state: " << (armed ? "armed" : "disarmed");
    LOG_DEBUG({"RecordingSession"}, msg.str());
}

void RecordingSession::replaceArmedLanes(const std::vector<std::string>& laneIds) {
    std::unique_lock<std::shared_mutex> lock(_armStateMutex);
    _armedLanes.clear();

    for (const auto& laneId : laneIds) {
        if (laneId.empty()) {
            continue;
        }
        _armedLanes[laneId].store(true, std::memory_order_release);
    }
}

bool RecordingSession::isLaneArmed(const std::string& laneId) const {
    std::shared_lock<std::shared_mutex> lock(_armStateMutex);
    auto it = _armedLanes.find(laneId);
    if (it == _armedLanes.end()) {
        return false;
    }
    return it->second.load(std::memory_order_acquire);
}

std::vector<std::string> RecordingSession::getArmedLaneIds() const {
    std::vector<std::string> laneIds;
    std::shared_lock<std::shared_mutex> lock(_armStateMutex);
    laneIds.reserve(_armedLanes.size());

    for (const auto& [laneId, armed] : _armedLanes) {
        if (armed.load(std::memory_order_acquire)) {
            laneIds.push_back(laneId);
        }
    }

    std::sort(laneIds.begin(), laneIds.end());
    return laneIds;
}

void RecordingSession::bindInputToLane(const std::string& inputNodeId, const std::string& laneId) {
    std::unique_lock<std::shared_mutex> lock(_inputMapMutex);
    _inputToLaneMap[inputNodeId] = laneId;
    std::ostringstream msg;
    msg << "Bound input " << inputNodeId << " to lane " << laneId;
    LOG_DEBUG({"RecordingSession"}, msg.str());
}

void RecordingSession::clearInputBindings() {
    std::unique_lock<std::shared_mutex> lock(_inputMapMutex);
    _inputToLaneMap.clear();
}

void RecordingSession::replaceInputBindings(
    const std::vector<std::pair<std::string, std::string>>& bindings
) {
    std::unique_lock<std::shared_mutex> lock(_inputMapMutex);
    _inputToLaneMap.clear();

    for (const auto& [inputNodeId, laneId] : bindings) {
        if (inputNodeId.empty() || laneId.empty()) {
            continue;
        }
        _inputToLaneMap[inputNodeId] = laneId;
    }
}

std::string RecordingSession::getTargetLaneForInput(const std::string& inputNodeId) const {
    std::shared_lock<std::shared_mutex> lock(_inputMapMutex);
    auto it = _inputToLaneMap.find(inputNodeId);
    if (it == _inputToLaneMap.end()) {
        return "";
    }
    return it->second;
}

bool RecordingSession::captureAudioChunk(const RecordedAudioChunk& chunk) {
    if (!_isRecording.load(std::memory_order_acquire)) {
        return false;
    }
    return _audioChunkQueue.push(chunk);
}

bool RecordingSession::captureMidiChunk(const RecordedMidiChunk& chunk) {
    if (!_isRecording.load(std::memory_order_acquire)) {
        return false;
    }
    return _midiChunkQueue.push(chunk);
}

size_t RecordingSession::consumeAudioChunks(std::vector<RecordedAudioChunk>& out) {
    size_t count = 0;
    RecordedAudioChunk chunk;
    while (_audioChunkQueue.pop(chunk)) {
        out.push_back(chunk);
        count++;
    }
    return count;
}

size_t RecordingSession::consumeMidiChunks(std::vector<RecordedMidiChunk>& out) {
    size_t count = 0;
    RecordedMidiChunk chunk;
    while (_midiChunkQueue.pop(chunk)) {
        out.push_back(chunk);
        count++;
    }
    return count;
}

bool RecordingSession::captureFinalOutput(
    const AudioBus& output,
    uint64_t blockStartSamples,
    const std::string& laneId
) {
    if (!_isRecording.load(std::memory_order_acquire)) {
        return false;
    }

    if (!isLaneArmed(laneId)) {
        return false;
    }

    // Convert interleaved AudioBus to RecordedAudioChunk
    RecordedAudioChunk chunk;
    chunk.laneId = laneId;
    chunk.numChannels = output.numChannels();
    chunk.sampleRate = 44100; // TODO: Get from context
    chunk.startSample = blockStartSamples;
    chunk.provisionalAssetId = "temp-master-" + std::to_string(blockStartSamples);

    // Copy interleaved data directly (AudioBus already provides interleaved format)
    const int numFrames = output.numFrames();
    chunk.interleaved.resize(chunk.numChannels * numFrames);
    const float* srcData = output.data();
    if (srcData) {
        std::memcpy(chunk.interleaved.data(), srcData, chunk.numChannels * numFrames * sizeof(float));
    }

    // Queue for async flush (lock-free)
    return _audioChunkQueue.push(chunk);
}
