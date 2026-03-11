#include "core/SignalApp.hpp"
#include "core/PluginCrashTracking.hpp"
#include "logging/Logging.hpp"
#include <exception>
#include <csignal>
#include <cstdlib>
#include <cstring>
#include <sstream>

void busErrorHandler(int sig, siginfo_t* info, void* context) {
    (void)context;
    (void)sig;
    LOG_CORE({"SignalApp"}, "SIGBUS (Bus Error) caught!");
    if (g_inPluginLoading && g_currentPluginPath[0] != '\0') {
        LOG_CORE({"SignalApp"}, std::string("Error occurred while loading plugin: ") + g_currentPluginPath);
        LOG_CORE({"SignalApp"}, "Skipping this plugin and continuing...");
    }
    std::ostringstream msg;
    msg << "Fault address: " << info->si_addr;
    LOG_CORE({"SignalApp"}, msg.str());

    // If we have a jump buffer set up, jump back to recover
    if (g_pluginLoadJumpSet) {
        siglongjmp(g_pluginLoadJumpBuf, 1);
    }

    // Otherwise, exit (shouldn't happen during plugin loading)
    std::_Exit(1);
}

void segfaultHandler(int sig, siginfo_t* info, void* context) {
    (void)context;
    (void)sig;
    LOG_CORE({"SignalApp"}, "SIGSEGV (Segmentation Fault) caught!");
    if (g_inPluginLoading && g_currentPluginPath[0] != '\0') {
        LOG_CORE({"SignalApp"}, std::string("Error occurred while loading plugin: ") + g_currentPluginPath);
        LOG_CORE({"SignalApp"}, "Skipping this plugin and continuing...");
    }
    std::ostringstream msg;
    msg << "Fault address: " << info->si_addr;
    LOG_CORE({"SignalApp"}, msg.str());

    // If we have a jump buffer set up, jump back to recover
    if (g_pluginLoadJumpSet) {
        siglongjmp(g_pluginLoadJumpBuf, 1);
    }

    // Otherwise, exit (shouldn't happen during plugin loading)
    std::_Exit(1);
}

int main() {
    // Set up signal handlers for bus errors and segfaults
    struct sigaction sa;
    memset(&sa, 0, sizeof(sa));
    sa.sa_sigaction = busErrorHandler;
    sa.sa_flags = SA_SIGINFO;
    sigemptyset(&sa.sa_mask);
    sigaction(SIGBUS, &sa, nullptr);

    memset(&sa, 0, sizeof(sa));
    sa.sa_sigaction = segfaultHandler;
    sa.sa_flags = SA_SIGINFO;
    sigemptyset(&sa.sa_mask);
    sigaction(SIGSEGV, &sa, nullptr);

    try {
        SignalApp app;
        return app.run();
    } catch (const std::exception& e) {
        // Use Core level for fatal errors during startup/shutdown
        LOG_CORE({"SignalApp"}, std::string("Fatal error: ") + e.what());
        return 1;
    } catch (...) {
        LOG_CORE({"SignalApp"}, "Unknown fatal error");
        return 1;
    }
}

