#include "backend/MiniaudioBackend.hpp"
#include "backend/OutputDeviceInfo.hpp"
#include "core/EngineRenderContext.hpp"
#include "core/AudioBus.hpp"
#include "logging/Logging.hpp"
#include <chrono>
#include <cstring>
#include <sstream>
#include <iomanip>

// Include miniaudio implementation
#define MINIAUDIO_IMPLEMENTATION
#include <miniaudio.h>

// Static callback wrapper - miniaudio requires a C-compatible function
void MiniaudioBackend::audioCallback(
    void* pDevice,
    void* pOutput,
    const void* pInput,
    unsigned int frameCount
) {
    // Get backend instance from user data
    ma_device* device = static_cast<ma_device*>(pDevice);
    MiniaudioBackend* backend = static_cast<MiniaudioBackend*>(device->pUserData);
    if (!backend) {
        // Safety: zero output if backend is null
        if (pOutput) {
            std::memset(pOutput, 0, frameCount * device->playback.channels * sizeof(float));
        }
        return;
    }

    // Cast buffers to float* (miniaudio uses float32 format)
    float* output = static_cast<float*>(pOutput);
    const float* input = pInput ? static_cast<const float*>(pInput) : nullptr;

    // Process audio
    backend->processAudio(output, input, frameCount);
}

MiniaudioBackend::MiniaudioBackend()
    : _initialised(false)
    , _running(false)
    , _context(nullptr)
    , _device(nullptr)
    , _actualSampleRate(0.0)
    , _actualBufferSize(0)
    , _actualOutputChannels(0)
    , _outputDeviceName("System Default")
    , _activeDeviceId("")
    , _hostTimeSeconds(0.0)
{
}

MiniaudioBackend::~MiniaudioBackend() {
    shutdown();
}

bool MiniaudioBackend::initialise(const AudioBackendConfig& config) {
    if (_initialised.load()) {
        LOG_DEBUG({"MiniaudioBackend"}, "Already initialised");
        return false;
    }

    _config = config;

    // Allocate context
    _context = new ma_context;
    ma_result result = ma_context_init(nullptr, 0, nullptr, static_cast<ma_context*>(_context));
    if (result != MA_SUCCESS) {
        std::ostringstream errMsg;
        errMsg << "Failed to initialise context: " << result;
        LOG_ERROR({"MiniaudioBackend"}, errMsg.str());
        delete static_cast<ma_context*>(_context);
        _context = nullptr;
        return false;
    }

    // Allocate device
    _device = new ma_device;
    std::memset(_device, 0, sizeof(ma_device));
    ma_device* device = static_cast<ma_device*>(_device);

    // Configure device
    ma_device_config deviceConfig = ma_device_config_init(ma_device_type_playback);
    deviceConfig.playback.format = ma_format_f32;  // 32-bit float
    deviceConfig.playback.channels = static_cast<ma_uint32>(config.numOutputChannels);
    deviceConfig.sampleRate = static_cast<ma_uint32>(config.preferredSampleRate);
    // Cast callback to match miniaudio's expected signature
    deviceConfig.dataCallback = reinterpret_cast<ma_device_data_proc>(audioCallback);
    deviceConfig.pUserData = this;  // Pass backend instance to callback

    // If a specific output device is requested, set it
    if (config.outputDeviceId.has_value()) {
        // Device selection will be handled after device enumeration
        // For now, continue with default device initialization
    }

    // Initialise device
    result = ma_device_init(static_cast<ma_context*>(_context), &deviceConfig, device);
    if (result != MA_SUCCESS) {
        std::ostringstream errMsg;
        errMsg << "Failed to initialise device: " << result;
        LOG_ERROR({"MiniaudioBackend"}, errMsg.str());
        ma_context_uninit(static_cast<ma_context*>(_context));
        delete static_cast<ma_context*>(_context);
        delete device;
        _context = nullptr;
        _device = nullptr;
        return false;
    }

    // Get actual runtime values (device may have negotiated different settings)
    _actualSampleRate.store(static_cast<double>(device->sampleRate), std::memory_order_release);
    _actualBufferSize.store(device->playback.internalPeriodSizeInFrames, std::memory_order_release);
    _actualOutputChannels.store(device->playback.channels, std::memory_order_release);

    // Try to get device name
    ma_device_info* playbackInfos = nullptr;
    ma_uint32 playbackCount = 0;
    ma_device_info* captureInfos = nullptr;
    ma_uint32 captureCount = 0;

    if (ma_context_get_devices(static_cast<ma_context*>(_context), &playbackInfos, &playbackCount, &captureInfos, &captureCount) == MA_SUCCESS) {
        // Find the default playback device
        ma_uint32 defaultPlaybackIndex = 0;
        for (ma_uint32 i = 0; i < playbackCount; ++i) {
            if (playbackInfos[i].isDefault) {
                defaultPlaybackIndex = i;
                break;
            }
        }

        if (defaultPlaybackIndex < playbackCount && playbackInfos[defaultPlaybackIndex].name[0] != '\0') {
            _outputDeviceName = std::string(playbackInfos[defaultPlaybackIndex].name);
            // Generate device ID: "miniaudio-<index>"
            std::ostringstream oss;
            oss << "miniaudio-" << defaultPlaybackIndex;
            _activeDeviceId = oss.str();
        }
    }

    _initialised.store(true, std::memory_order_release);

    std::ostringstream msg;
    msg << "Initialised: sampleRate=" << _actualSampleRate.load()
        << ", bufferSize=" << _actualBufferSize.load()
        << ", outChannels=" << _actualOutputChannels.load()
        << ", device=" << _outputDeviceName;
    LOG_INFO({"MiniaudioBackend"}, msg.str());

    return true;
}

void MiniaudioBackend::shutdown() {
    stop();

    if (_device) {
        ma_device_uninit(static_cast<ma_device*>(_device));
        delete static_cast<ma_device*>(_device);
        _device = nullptr;
    }

    if (_context) {
        ma_context_uninit(static_cast<ma_context*>(_context));
        delete static_cast<ma_context*>(_context);
        _context = nullptr;
    }

    _initialised.store(false, std::memory_order_release);
    _renderCallback = nullptr;

    LOG_INFO({"MiniaudioBackend"}, "Shutdown complete");
}

bool MiniaudioBackend::start() {
    if (!_initialised.load()) {
        LOG_WARN({"MiniaudioBackend"}, "Cannot start: not initialised");
        return false;
    }

    if (_running.load()) {
        LOG_DEBUG({"MiniaudioBackend"}, "Already running");
        return false;
    }

    if (!_renderCallback) {
        LOG_WARN({"MiniaudioBackend"}, "Cannot start: no render callback set");
        return false;
    }

    ma_result result = ma_device_start(static_cast<ma_device*>(_device));
    if (result != MA_SUCCESS) {
        std::ostringstream errMsg;
        errMsg << "Failed to start device: " << result;
        LOG_ERROR({"MiniaudioBackend"}, errMsg.str());
        return false;
    }

    _running.store(true, std::memory_order_release);
    _hostTimeSeconds.store(0.0, std::memory_order_release);

    LOG_INFO({"MiniaudioBackend"}, "Started");
    return true;
}

void MiniaudioBackend::stop() {
    if (!_running.load()) {
        return;
    }

    if (_device) {
        ma_device_stop(static_cast<ma_device*>(_device));
    }

    _running.store(false, std::memory_order_release);

    LOG_INFO({"MiniaudioBackend"}, "Stopped");
}

void MiniaudioBackend::setRenderCallback(RenderCallback callback) {
    _renderCallback = std::move(callback);
}

double MiniaudioBackend::getSampleRate() const {
    return _actualSampleRate.load(std::memory_order_acquire);
}

int MiniaudioBackend::getBufferSize() const {
    return static_cast<int>(_actualBufferSize.load(std::memory_order_acquire));
}

int MiniaudioBackend::getNumInputChannels() const {
    return _config.numInputChannels;
}

int MiniaudioBackend::getNumOutputChannels() const {
    return static_cast<int>(_actualOutputChannels.load(std::memory_order_acquire));
}

std::string MiniaudioBackend::getOutputDeviceName() const {
    return _outputDeviceName;
}

void MiniaudioBackend::processAudio(
    float* output,
    const float* input,
    unsigned int frameCount
) {
    // Real-time safety: No allocations, locks, or I/O in this function

    if (!_renderCallback || !output) {
        // Safety: zero output if callback not set
        std::memset(output, 0, frameCount * _actualOutputChannels.load(std::memory_order_acquire) * sizeof(float));
        return;
    }

    // Update host time (monotonic, in seconds)
    // Use a simple counter-based approach for now (can be improved with ma_device_get_time)
    static std::atomic<uint64_t> frameCounter(0);
    uint64_t currentFrame = frameCounter.fetch_add(frameCount, std::memory_order_acq_rel);
    double sampleRate = _actualSampleRate.load(std::memory_order_acquire);
    double hostTime = sampleRate > 0.0 ? (static_cast<double>(currentFrame) / sampleRate) : 0.0;
    _hostTimeSeconds.store(hostTime, std::memory_order_release);

    // Create render context
    EngineRenderContext ctx;
    ctx.hostTimeSeconds = hostTime;
    ctx.sampleRate = sampleRate;
    ctx.blockSize = static_cast<int>(frameCount);
    ctx.playheadSamples = 0;  // Will be updated by EngineHost

    // Wrap input buffer (if available)
    AudioBus inputBus(
        const_cast<float*>(input),  // AudioBus doesn't modify, but needs non-const for interface
        _config.numInputChannels,
        static_cast<int>(frameCount),
        true  // read-only
    );

    // Wrap output buffer
    int numOutputChannels = static_cast<int>(_actualOutputChannels.load(std::memory_order_acquire));
    AudioBus outputBus(
        output,
        numOutputChannels,
        static_cast<int>(frameCount),
        false  // writable
    );

    // Call render callback
    try {
        _renderCallback(ctx, inputBus, outputBus);
    } catch (...) {
        // Safety: Never throw from audio callback
        // Zero output on error
        std::memset(output, 0, frameCount * numOutputChannels * sizeof(float));
    }
}

std::string MiniaudioBackend::getActiveOutputDeviceId() const {
    return _activeDeviceId;
}

std::vector<OutputDeviceInfo> MiniaudioBackend::enumerateOutputDevices() const {
    std::vector<OutputDeviceInfo> devices;

    if (!_context) {
        // Context not initialised - try to create a temporary one for enumeration
        ma_context tempContext;
        ma_result result = ma_context_init(nullptr, 0, nullptr, &tempContext);
        if (result != MA_SUCCESS) {
            std::ostringstream errMsg;
            errMsg << "Failed to create context for enumeration: " << result;
            LOG_ERROR({"MiniaudioBackend"}, errMsg.str());
            return devices;
        }

        ma_device_info* playbackInfos = nullptr;
        ma_uint32 playbackCount = 0;
        ma_device_info* captureInfos = nullptr;
        ma_uint32 captureCount = 0;

        if (ma_context_get_devices(&tempContext, &playbackInfos, &playbackCount, &captureInfos, &captureCount) == MA_SUCCESS) {
            std::ostringstream msg;
            msg << "Enumerated " << playbackCount << " playback devices";
            LOG_DEBUG({"MiniaudioBackend"}, msg.str());
            for (ma_uint32 i = 0; i < playbackCount; ++i) {
                OutputDeviceInfo info;
                // Generate stable ID: "miniaudio-<index>"
                std::ostringstream oss;
                oss << "miniaudio-" << i;
                info.id = oss.str();
                info.name = playbackInfos[i].name[0] != '\0' ? std::string(playbackInfos[i].name) : "Unknown Device";
                info.isDefault = playbackInfos[i].isDefault != 0;
                // Get format info if available
                if (playbackInfos[i].nativeDataFormatCount > 0) {
                    info.maxChannels = playbackInfos[i].nativeDataFormats[0].channels;
                    info.preferredSampleRate = playbackInfos[i].nativeDataFormats[0].sampleRate;
                } else {
                    // Fallback values if no format info available
                    info.maxChannels = 2; // Assume stereo
                    info.preferredSampleRate = 44100; // Assume 44.1kHz
                }
                devices.push_back(info);
            }
        }

        ma_context_uninit(&tempContext);
    } else {
        // Use existing context
        ma_device_info* playbackInfos = nullptr;
        ma_uint32 playbackCount = 0;
        ma_device_info* captureInfos = nullptr;
        ma_uint32 captureCount = 0;

        if (ma_context_get_devices(static_cast<ma_context*>(_context), &playbackInfos, &playbackCount, &captureInfos, &captureCount) == MA_SUCCESS) {
            std::ostringstream msg;
            msg << "Enumerated " << playbackCount << " playback devices";
            LOG_DEBUG({"MiniaudioBackend"}, msg.str());
            for (ma_uint32 i = 0; i < playbackCount; ++i) {
                OutputDeviceInfo info;
                // Generate stable ID: "miniaudio-<index>"
                std::ostringstream oss;
                oss << "miniaudio-" << i;
                info.id = oss.str();
                info.name = playbackInfos[i].name[0] != '\0' ? std::string(playbackInfos[i].name) : "Unknown Device";
                info.isDefault = playbackInfos[i].isDefault != 0;
                // Get format info if available
                if (playbackInfos[i].nativeDataFormatCount > 0) {
                    info.maxChannels = playbackInfos[i].nativeDataFormats[0].channels;
                    info.preferredSampleRate = playbackInfos[i].nativeDataFormats[0].sampleRate;
                } else {
                    // Fallback values if no format info available
                    info.maxChannels = 2; // Assume stereo
                    info.preferredSampleRate = 44100; // Assume 44.1kHz
                }
                devices.push_back(info);
            }
        }
    }

    return devices;
}

bool MiniaudioBackend::setOutputDevice(const std::string& deviceId) {
    if (deviceId == _activeDeviceId) {
        // Already using this device - no-op
        return true;
    }

    // Parse device ID to get index (format: "miniaudio-<index>")
    if (deviceId.find("miniaudio-") != 0) {
        LOG_ERROR({"MiniaudioBackend"}, std::string("Invalid device ID format: ") + deviceId);
        return false;
    }

    std::string indexStr = deviceId.substr(10); // Skip "miniaudio-"
    unsigned int deviceIndex = 0;
    try {
        deviceIndex = std::stoul(indexStr);
    } catch (...) {
        LOG_ERROR({"MiniaudioBackend"}, std::string("Invalid device index in ID: ") + deviceId);
        return false;
    }

    // Check if device exists
    auto devices = enumerateOutputDevices();
    if (deviceIndex >= devices.size()) {
        std::ostringstream errMsg;
        errMsg << "Device index out of range: " << deviceIndex;
        LOG_ERROR({"MiniaudioBackend"}, errMsg.str());
        return false;
    }

    // Stop current device if running
    bool wasRunning = _running.load();
    if (wasRunning) {
        stop();
    }

    // Shutdown current device
    if (_device) {
        ma_device_uninit(static_cast<ma_device*>(_device));
        delete static_cast<ma_device*>(_device);
        _device = nullptr;
    }

    // Reinitialise with new device
    AudioBackendConfig newConfig = _config;
    newConfig.outputDeviceId = deviceId; // Store the ID for reference

    // Allocate new device
    _device = new ma_device;
    std::memset(_device, 0, sizeof(ma_device));
    ma_device* device = static_cast<ma_device*>(_device);

    // Configure device
    ma_device_config deviceConfig = ma_device_config_init(ma_device_type_playback);
    deviceConfig.playback.format = ma_format_f32;
    deviceConfig.playback.channels = static_cast<ma_uint32>(_config.numOutputChannels);
    deviceConfig.sampleRate = static_cast<ma_uint32>(_config.preferredSampleRate);
    deviceConfig.dataCallback = reinterpret_cast<ma_device_data_proc>(audioCallback);
    deviceConfig.pUserData = this;

    // Set specific device by index
    ma_device_info* playbackInfos = nullptr;
    ma_uint32 playbackCount = 0;
    ma_device_info* captureInfos = nullptr;
    ma_uint32 captureCount = 0;

    if (ma_context_get_devices(static_cast<ma_context*>(_context), &playbackInfos, &playbackCount, &captureInfos, &captureCount) != MA_SUCCESS) {
        LOG_ERROR({"MiniaudioBackend"}, "Failed to get device list for selection");
        delete device;
        _device = nullptr;
        return false;
    }

    if (deviceIndex >= playbackCount) {
        std::ostringstream errMsg;
        errMsg << "Device index out of range: " << deviceIndex;
        LOG_ERROR({"MiniaudioBackend"}, errMsg.str());
        delete device;
        _device = nullptr;
        return false;
    }

    // Select the specific device
    deviceConfig.playback.pDeviceID = &playbackInfos[deviceIndex].id;

    // Initialise device
    ma_result result = ma_device_init(static_cast<ma_context*>(_context), &deviceConfig, device);
    if (result != MA_SUCCESS) {
        std::ostringstream errMsg;
        errMsg << "Failed to initialise device " << deviceId << ": " << result;
        LOG_ERROR({"MiniaudioBackend"}, errMsg.str());
        delete device;
        _device = nullptr;
        // Try to restore previous device if possible
        if (wasRunning) {
            // Reinitialise with default device
            initialise(_config);
            if (wasRunning) {
                start();
            }
        }
        return false;
    }

    // Update runtime values
    _actualSampleRate.store(static_cast<double>(device->sampleRate), std::memory_order_release);
    _actualBufferSize.store(device->playback.internalPeriodSizeInFrames, std::memory_order_release);
    _actualOutputChannels.store(device->playback.channels, std::memory_order_release);
    _outputDeviceName = devices[deviceIndex].name;
    _activeDeviceId = deviceId;

    // Restart if it was running
    if (wasRunning) {
        if (!start()) {
            LOG_ERROR({"MiniaudioBackend"}, "Failed to restart device after selection");
            return false;
        }
    }

    std::ostringstream msg;
    msg << "Switched to device: " << _outputDeviceName << " (ID: " << deviceId << ")";
    LOG_INFO({"MiniaudioBackend"}, msg.str());
    return true;
}
