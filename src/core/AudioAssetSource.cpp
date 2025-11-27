#include "core/AudioAssetSource.hpp"
#include <iostream>
#include <fstream>
#include <cstring>
#include <cmath>

// Include miniaudio for decoding (implementation already in MiniaudioBackend.cpp)
#include <miniaudio.h>

FileAudioAssetSource::FileAudioAssetSource() {
}

FileAudioAssetSource::~FileAudioAssetSource() {
    // Release all assets
    _assets.clear();
}

bool FileAudioAssetSource::prepareAsset(const AssetId& assetId, const AudioAssetMetadata& metadata) {
    // Check if already prepared
    if (_assets.find(assetId) != _assets.end()) {
        std::cout << "[FileAudioAssetSource] Asset already prepared: " << assetId << std::endl;
        return true;
    }

    // Skip test assets (handled by stub)
    if (assetId.find("test://") == 0) {
        std::cout << "[FileAudioAssetSource] Skipping test asset: " << assetId << std::endl;
        return false;
    }

    std::cout << "[FileAudioAssetSource] Preparing asset: " << assetId << " from " << metadata.path << std::endl;

    // Use miniaudio to decode the file
    ma_decoder_config decoderConfig = ma_decoder_config_init(ma_format_f32, 0, 0); // float32, default channels, default sample rate
    ma_decoder decoder;
    ma_result result = ma_decoder_init_file(metadata.path.c_str(), &decoderConfig, &decoder);
    if (result != MA_SUCCESS) {
        std::cerr << "[FileAudioAssetSource] Failed to open file: " << metadata.path
                  << " (error: " << result << ")" << std::endl;
        return false;
    }

    // Get decoder output format from the decoder structure
    ma_format format = decoder.outputFormat;
    ma_uint32 channels = decoder.outputChannels;
    ma_uint32 sampleRate = decoder.outputSampleRate;

    // Decode file in chunks to determine total frame count and decode all data
    // We'll decode into a temporary buffer first, then copy to final buffer
    std::vector<float> tempPcm;
    const ma_uint32 chunkSize = 16384; // 16k frames per chunk
    ma_uint64 totalFrames = 0;

    while (true) {
        std::vector<float> chunk(chunkSize * channels);
        ma_uint64 framesRead = 0;
        result = ma_decoder_read_pcm_frames(&decoder, chunk.data(), chunkSize, &framesRead);

        if (result != MA_SUCCESS || framesRead == 0) {
            break;
        }

        // Append chunk to temp buffer
        size_t oldSize = tempPcm.size();
        tempPcm.resize(oldSize + framesRead * channels);
        std::memcpy(tempPcm.data() + oldSize, chunk.data(), framesRead * channels * sizeof(float));

        totalFrames += framesRead;

        // Safety check: if we've read a lot, stop (file might be corrupt or very large)
        if (totalFrames > 100000000) { // ~100M frames at 44.1kHz = ~38 minutes
            std::cerr << "[FileAudioAssetSource] File appears too long, stopping decode" << std::endl;
            break;
        }
    }

    ma_uint64 frameCount = totalFrames;

    if (frameCount == 0) {
        std::cerr << "[FileAudioAssetSource] Failed to decode any frames from file: " << metadata.path << std::endl;
        ma_decoder_uninit(&decoder);
        return false;
    }

    // Validate format (we need float32)
    if (format != ma_format_f32) {
        std::cerr << "[FileAudioAssetSource] Unsupported format (expected float32): " << format << std::endl;
        ma_decoder_uninit(&decoder);
        return false;
    }

    // Check file size limit (safety check)
    uint64_t estimatedSizeMB = (frameCount * channels * sizeof(float)) / (1024 * 1024);
    if (estimatedSizeMB > MAX_FILE_SIZE_MB) {
        std::cerr << "[FileAudioAssetSource] File too large: " << estimatedSizeMB
                  << " MB (limit: " << MAX_FILE_SIZE_MB << " MB)" << std::endl;
        ma_decoder_uninit(&decoder);
        return false;
    }

    // Clean up decoder (we've already decoded everything)
    ma_decoder_uninit(&decoder);

    // Store decoded data (tempPcm already contains all decoded frames)
    AssetData assetData;
    assetData.pcm = std::move(tempPcm);
    assetData.channels = channels;
    assetData.sampleRate = sampleRate;
    assetData.frames = frameCount;

    _assets[assetId] = std::move(assetData);

    std::cout << "[FileAudioAssetSource] Asset prepared: " << assetId
              << " (" << channels << " channels, " << sampleRate << " Hz, "
              << frameCount << " frames)" << std::endl;

    return true;
}

void FileAudioAssetSource::releaseAsset(const AssetId& assetId) {
    auto it = _assets.find(assetId);
    if (it != _assets.end()) {
        _assets.erase(it);
        std::cout << "[FileAudioAssetSource] Released asset: " << assetId << std::endl;
    }
}

bool FileAudioAssetSource::readSamples(
    const AssetId& assetId,
    uint64_t startSample,
    int numFrames,
    AudioBuffer& buffer,
    int destFrameOffset,
    int numChannels
) {
    auto it = _assets.find(assetId);
    if (it == _assets.end()) {
        // Asset not found - produce silence
        for (int frame = 0; frame < numFrames; ++frame) {
            for (int ch = 0; ch < numChannels && ch < buffer.numChannels(); ++ch) {
                int destFrame = destFrameOffset + frame;
                if (destFrame >= 0 && destFrame < buffer.numFrames()) {
                    buffer.setSample(destFrame, ch, 0.0f);
                }
            }
        }
        return false;
    }

    const AssetData& asset = it->second;

    // Clamp startSample to valid range
    if (startSample >= asset.frames) {
        // Beyond end of asset - produce silence
        for (int frame = 0; frame < numFrames; ++frame) {
            for (int ch = 0; ch < numChannels && ch < buffer.numChannels(); ++ch) {
                int destFrame = destFrameOffset + frame;
                if (destFrame >= 0 && destFrame < buffer.numFrames()) {
                    buffer.setSample(destFrame, ch, 0.0f);
                }
            }
        }
        return true;
    }

    // Calculate how many frames we can actually read
    uint64_t availableFrames = asset.frames - startSample;
    int framesToRead = static_cast<int>(std::min(static_cast<uint64_t>(numFrames), availableFrames));

    // Copy samples from interleaved PCM to deinterleaved buffer
    for (int frame = 0; frame < framesToRead; ++frame) {
        uint64_t sourceFrame = startSample + frame;
        int destFrame = destFrameOffset + frame;

        if (destFrame < 0 || destFrame >= buffer.numFrames()) {
            continue;
        }

        // Copy from interleaved PCM (asset.channels) to deinterleaved buffer (numChannels)
        for (int ch = 0; ch < numChannels && ch < buffer.numChannels(); ++ch) {
            // Map channel (handle mono -> stereo, etc.)
            int sourceCh = std::min(ch, static_cast<int>(asset.channels) - 1);
            size_t pcmIndex = sourceFrame * asset.channels + sourceCh;
            float sample = asset.pcm[pcmIndex];
            buffer.setSample(destFrame, ch, sample);
        }
    }

    // Zero-pad remaining frames if we didn't read enough
    for (int frame = framesToRead; frame < numFrames; ++frame) {
        int destFrame = destFrameOffset + frame;
        if (destFrame >= 0 && destFrame < buffer.numFrames()) {
            for (int ch = 0; ch < numChannels && ch < buffer.numChannels(); ++ch) {
                buffer.setSample(destFrame, ch, 0.0f);
            }
        }
    }

    return true;
}

AudioAssetSourceRouter::AudioAssetSourceRouter()
    : _stubSource(std::make_unique<StubAudioAssetSource>())
    , _fileSource(std::make_unique<FileAudioAssetSource>())
{
}

void AudioAssetSourceRouter::setSampleRate(double sampleRate) {
    _stubSource->setSampleRate(sampleRate);
}

bool AudioAssetSourceRouter::prepareAsset(const AssetId& assetId, const AudioAssetMetadata& metadata) {
    // Test assets are handled by stub source (no preparation needed)
    if (assetId.find("test://") == 0) {
        return true;
    }

    // File assets are handled by file source
    return _fileSource->prepareAsset(assetId, metadata);
}

void AudioAssetSourceRouter::releaseAsset(const AssetId& assetId) {
    // Test assets don't need release
    if (assetId.find("test://") == 0) {
        return;
    }

    _fileSource->releaseAsset(assetId);
}

bool AudioAssetSourceRouter::readSamples(
    const AssetId& assetId,
    uint64_t startSample,
    int numFrames,
    AudioBuffer& buffer,
    int destFrameOffset,
    int numChannels
) {
    // Route to appropriate source based on asset ID
    if (assetId.find("test://") == 0) {
        return _stubSource->readSamples(assetId, startSample, numFrames, buffer, destFrameOffset, numChannels);
    } else {
        return _fileSource->readSamples(assetId, startSample, numFrames, buffer, destFrameOffset, numChannels);
    }
}

