#include "core/PluginHost.hpp"
#include "clap/ClapPluginInstance.hpp"
#include "clap/ClapRegistry.hpp"
#include "clap/clap.h"
#include "logging/Logging.hpp"
#include "vst3/Vst3Backend.hpp"
#include <sstream>

PluginHost::PluginHost() {
    _clapRegistry = std::make_unique<ClapRegistry>();
    _vst3Backend = std::make_unique<Vst3Backend>();
    // Phase 5: Defer plugin scanning until after server starts
    // This prevents Signal from crashing before it can accept connections
    LOG_INFO({"PluginHost"}, "Created (plugin scanning deferred, CLAP + VST3 scaffold)");
}

PluginHost::~PluginHost() {
    LOG_DEBUG({"PluginHost"}, "Destroyed");
}

void PluginHost::scanPlugins(std::stop_token stopToken) {
    // Phase 5+: Scan for plugins after server starts
    // Wrap in try-catch to prevent crashes from bad plugins
    LOG_INFO({"PluginHost"}, "Starting plugin scan...");

    {
        std::lock_guard<std::mutex> lock(scanMutex_);
        scanStatus_ = PluginScanStatus{
            .state = PluginScanState::Running,
            .plugin_count = 0,
            .last_error = std::nullopt,
            .duration = std::nullopt,
        };
    }

    const auto start = std::chrono::steady_clock::now();

    try {
        {
            std::lock_guard<std::mutex> lock(registryMutex_);
            _clapRegistry->scanDefaultPaths(stopToken);
            _vst3Backend->scanDefaultPaths(stopToken);
        }

        const auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::steady_clock::now() - start
        );

        size_t pluginCount = 0;
        size_t clapCount = 0;
        size_t vst3Count = 0;
        {
            std::lock_guard<std::mutex> lock(registryMutex_);
            clapCount = _clapRegistry->listPlugins().size();
            vst3Count = _vst3Backend->listPlugins().size();
            pluginCount = clapCount + vst3Count;
        }

        {
            std::lock_guard<std::mutex> lock(scanMutex_);
            scanStatus_.state = stopToken.stop_requested() ? PluginScanState::Cancelled : PluginScanState::Completed;
            scanStatus_.plugin_count = static_cast<std::uint32_t>(pluginCount);
            scanStatus_.duration = duration;
        }

        std::ostringstream msg;
        msg << "Plugin scan " << (stopToken.stop_requested() ? "cancelled" : "complete")
            << " - found " << pluginCount << " plugin(s) [CLAP=" << clapCount
            << ", VST3=" << vst3Count << "]";
        LOG_INFO({"PluginHost"}, msg.str());
    } catch (const std::exception& e) {
        LOG_ERROR({"PluginHost"}, std::string("Exception during plugin scanning: ") + e.what());

        const auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::steady_clock::now() - start
        );

        size_t pluginCount = 0;
        {
            std::lock_guard<std::mutex> lock(registryMutex_);
            pluginCount = _clapRegistry->listPlugins().size() + _vst3Backend->listPlugins().size();
        }

        {
            std::lock_guard<std::mutex> lock(scanMutex_);
            scanStatus_.state = PluginScanState::Failed;
            scanStatus_.plugin_count = static_cast<std::uint32_t>(pluginCount);
            scanStatus_.last_error = e.what();
            scanStatus_.duration = duration;
        }

        std::ostringstream msg;
        msg << "Continuing with " << pluginCount << " successfully loaded plugins";
        LOG_INFO({"PluginHost"}, msg.str());
    } catch (...) {
        LOG_ERROR({"PluginHost"}, "Unknown exception during plugin scanning, continuing anyway");

        const auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(
            std::chrono::steady_clock::now() - start
        );

        size_t pluginCount = 0;
        {
            std::lock_guard<std::mutex> lock(registryMutex_);
            pluginCount = _clapRegistry->listPlugins().size() + _vst3Backend->listPlugins().size();
        }

        {
            std::lock_guard<std::mutex> lock(scanMutex_);
            scanStatus_.state = PluginScanState::Failed;
            scanStatus_.plugin_count = static_cast<std::uint32_t>(pluginCount);
            scanStatus_.last_error = "Unknown exception during plugin scanning";
            scanStatus_.duration = duration;
        }

        std::ostringstream msg;
        msg << "Continuing with " << pluginCount << " successfully loaded plugins";
        LOG_INFO({"PluginHost"}, msg.str());
    }
}

PluginHost::PluginScanStatus PluginHost::scanStatus() const {
    std::lock_guard<std::mutex> lock(scanMutex_);
    return scanStatus_;
}

std::unique_ptr<PluginInstance> PluginHost::createInstance(const PluginDescriptor& desc) {
    std::ostringstream msg;
    msg << "Creating instance for plugin: " << desc.id << " (format: " << static_cast<int>(desc.format) << ")";
    LOG_DEBUG({"PluginHost"}, msg.str());

    std::lock_guard<std::mutex> lock(registryMutex_);

    switch (desc.format) {
        case PluginFormat::Clap:
            {
                // Phase 5: Use registry to find plugin
                auto library = _clapRegistry->getLibrary(desc.id);
                if (!library) {
                    LOG_ERROR({"PluginHost"}, std::string("Plugin not found in registry: ") + desc.id);
                    return nullptr;
                }

                const clap_plugin_descriptor* clapDesc = library->getDescriptor(desc.id.c_str());
                if (!clapDesc) {
                    LOG_ERROR({"PluginHost"}, std::string("CLAP descriptor not found for: ") + desc.id);
                    return nullptr;
                }

                auto instance = createClapInstance(library, clapDesc);
                if (instance) {
                    LOG_DEBUG({"PluginHost"}, std::string("Successfully created CLAP instance: ") + desc.id);
                } else {
                    LOG_ERROR({"PluginHost"}, std::string("Failed to create CLAP instance: ") + desc.id);
                }
                return instance;
            }
        case PluginFormat::Vst3:
            {
                std::string error;
                auto instance = _vst3Backend->createInstance(desc, error);
                if (instance) {
                    LOG_DEBUG({"PluginHost"}, std::string("Successfully created VST3 instance: ") + desc.id);
                    return instance;
                }

                LOG_ERROR({"PluginHost"}, std::string("Failed to create VST3 instance: ") + error);
                return nullptr;
            }
        case PluginFormat::Au:
        case PluginFormat::Lv2:
        case PluginFormat::Native:
            // Not implemented in Phase 5
            LOG_ERROR({"PluginHost"}, std::string("Plugin format not yet supported: ") + std::to_string(static_cast<int>(desc.format)));
            return nullptr;
        default:
            LOG_ERROR({"PluginHost"}, std::string("Unknown plugin format: ") + std::to_string(static_cast<int>(desc.format)));
            return nullptr;
    }
}

bool PluginHost::isFormatSupported(PluginFormat format) const {
    if (format == PluginFormat::Clap) {
        return true;
    }

    if (format == PluginFormat::Vst3) {
        return _vst3Backend && _vst3Backend->isEnabled();
    }

    return false;
}
