#include <catch2/catch_test_macros.hpp>
#include "vst3/Vst3Registry.hpp"
#include <chrono>
#include <filesystem>
#include <fstream>
#include <string>

namespace {
class ScopedTempDir {
public:
    ScopedTempDir() {
        const auto base = std::filesystem::temp_directory_path();
        const auto nonce = std::chrono::steady_clock::now().time_since_epoch().count();
        _path = base / std::filesystem::path("loophole-signal-vst3-test-" + std::to_string(nonce));
        std::filesystem::create_directories(_path);
    }

    ~ScopedTempDir() {
        std::error_code ignored;
        std::filesystem::remove_all(_path, ignored);
    }

    const std::filesystem::path& path() const {
        return _path;
    }

private:
    std::filesystem::path _path;
};
}

TEST_CASE("VST3 registry scans directories and emits stable ids", "[plugin][vst3][registry]") {
    ScopedTempDir temp;
    const auto vst3Path = temp.path() / "TestPlugin.vst3";
    const auto ignoredPath = temp.path() / "readme.txt";

    std::ofstream(vst3Path.string()) << "dummy";
    std::ofstream(ignoredPath.string()) << "not a plugin";

    Vst3Registry registry;
    registry.scanPath(temp.path());

    const auto plugins = registry.listPlugins();
    REQUIRE(plugins.size() == 1);
    REQUIRE(plugins[0].format == PluginFormat::Vst3);
    REQUIRE(plugins[0].name == "TestPlugin");
    REQUIRE(plugins[0].id.rfind("vst3:", 0) == 0);

    const auto* descriptor = registry.findPluginById(plugins[0].id);
    REQUIRE(descriptor != nullptr);
    REQUIRE(descriptor->id == plugins[0].id);

    const auto pluginPath = registry.findPathById(plugins[0].id);
    REQUIRE(pluginPath.has_value());
    REQUIRE(pluginPath.value() == vst3Path);
}
