#pragma once

/// PluginHost - Factory for creating plugin instances
///
/// Thread: Control thread only
/// Ownership: Owned by EngineHost
///
/// This class is responsible for creating PluginInstance objects from PluginDescriptors.
/// Phase 5: Uses ClapRegistry for plugin discovery and creation.

#include "core/PluginInstance.hpp"
#include <chrono>
#include <memory>
#include <mutex>
#include <optional>
#include <stop_token>
#include <string>

// Forward declarations
class ClapRegistry;
class Vst3Backend;
std::unique_ptr<PluginInstance> createClapInstance(
    std::shared_ptr<class ClapPluginLibrary> library,
    const struct clap_plugin_descriptor* clapDesc
);

class PluginHost {
public:
    enum class PluginScanState {
        NotStarted,
        Running,
        Completed,
        Failed,
        Cancelled,
    };

    struct PluginScanStatus {
        PluginScanState state{PluginScanState::NotStarted};
        std::uint32_t plugin_count{0};
        std::uint32_t clap_plugin_count{0};
        std::uint32_t vst3_plugin_count{0};
        std::optional<std::string> last_error;
        std::optional<std::chrono::milliseconds> duration;

        const char* state_tag() const noexcept {
            switch (state) {
            case PluginScanState::NotStarted:
                return "notStarted";
            case PluginScanState::Running:
                return "running";
            case PluginScanState::Completed:
                return "completed";
            case PluginScanState::Failed:
                return "failed";
            case PluginScanState::Cancelled:
                return "cancelled";
            }
            return "unknown";
        }
    };

    PluginHost();
    virtual ~PluginHost();

    /// Create a plugin instance from a descriptor
    /// @param desc Plugin descriptor
    /// @return Plugin instance, or nullptr if creation failed or format not supported
    virtual std::unique_ptr<PluginInstance> createInstance(const PluginDescriptor& desc);

    /// Check if a plugin format is supported
    /// @param format Plugin format
    /// @return true if format is supported, false otherwise
    virtual bool isFormatSupported(PluginFormat format) const;

    /// Get the CLAP registry (for discovery)
    ClapRegistry& getClapRegistry() { return *_clapRegistry; }
    const ClapRegistry& getClapRegistry() const { return *_clapRegistry; }
    Vst3Backend& getVst3Backend() { return *_vst3Backend; }
    const Vst3Backend& getVst3Backend() const { return *_vst3Backend; }

    /// Scan for CLAP plugins (deferred from construction)
    /// This should be called after Signal has started its server
    /// to prevent crashes from blocking startup
    void scanPlugins(std::stop_token stopToken = {});

    PluginScanStatus scanStatus() const;
    std::vector<PluginDescriptor> listPlugins() const;

private:
    std::unique_ptr<ClapRegistry> _clapRegistry;
    std::unique_ptr<Vst3Backend> _vst3Backend;
    mutable std::mutex registryMutex_;
    mutable std::mutex scanMutex_;
    PluginScanStatus scanStatus_{};
};
