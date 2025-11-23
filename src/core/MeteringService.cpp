#include "core/MeteringService.hpp"
#include <algorithm>

MeteringService::MeteringService() {
}

MeteringService::~MeteringService() {
}

void MeteringService::registerChannel(const std::string& channelId) {
    std::lock_guard<std::mutex> lock(_mutex);
    if (_metering.find(channelId) == _metering.end()) {
        _metering[channelId] = std::make_unique<AtomicChannelMetering>(channelId);
    }
}

void MeteringService::unregisterChannel(const std::string& channelId) {
    std::lock_guard<std::mutex> lock(_mutex);
    _metering.erase(channelId);
}

AtomicChannelMetering* MeteringService::getAtomicMetering(const std::string& channelId) {
    std::lock_guard<std::mutex> lock(_mutex);
    auto it = _metering.find(channelId);
    if (it != _metering.end()) {
        return it->second.get();
    }
    return nullptr;
}

std::vector<ChannelMetering> MeteringService::snapshotAll() const {
    std::lock_guard<std::mutex> lock(_mutex);
    std::vector<ChannelMetering> result;
    result.reserve(_metering.size());

    for (const auto& pair : _metering) {
        result.push_back(pair.second->snapshot());
    }

    return result;
}

std::optional<ChannelMetering> MeteringService::snapshotChannel(const std::string& channelId) const {
    std::lock_guard<std::mutex> lock(_mutex);
    auto it = _metering.find(channelId);
    if (it != _metering.end()) {
        return it->second->snapshot();
    }
    return std::nullopt;
}


