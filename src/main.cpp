#include "core/SignalApp.hpp"
#include "core/PluginCrashTracking.hpp"
#include <iostream>
#include <exception>
#include <csignal>
#include <cstdlib>
#include <cstring>

void busErrorHandler(int sig, siginfo_t* info, void* context) {
    (void)context;
    std::cerr << std::endl;
    std::cerr << "[Signal] SIGBUS (Bus Error) caught!" << std::endl;
    if (g_inPluginLoading && g_currentPluginPath[0] != '\0') {
        std::cerr << "[Signal] Error occurred while loading plugin: " << g_currentPluginPath << std::endl;
        std::cerr << "[Signal] Skipping this plugin and continuing..." << std::endl;
    }
    std::cerr << "[Signal] Fault address: " << info->si_addr << std::endl;
    std::cerr.flush();

    // If we have a jump buffer set up, jump back to recover
    if (g_pluginLoadJumpSet) {
        siglongjmp(g_pluginLoadJumpBuf, 1);
    }

    // Otherwise, exit (shouldn't happen during plugin loading)
    std::_Exit(1);
}

void segfaultHandler(int sig, siginfo_t* info, void* context) {
    (void)context;
    std::cerr << std::endl;
    std::cerr << "[Signal] SIGSEGV (Segmentation Fault) caught!" << std::endl;
    if (g_inPluginLoading && g_currentPluginPath[0] != '\0') {
        std::cerr << "[Signal] Error occurred while loading plugin: " << g_currentPluginPath << std::endl;
        std::cerr << "[Signal] Skipping this plugin and continuing..." << std::endl;
    }
    std::cerr << "[Signal] Fault address: " << info->si_addr << std::endl;
    std::cerr.flush();

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
        std::cerr << "[Signal] Fatal error: " << e.what() << std::endl;
        return 1;
    } catch (...) {
        std::cerr << "[Signal] Unknown fatal error" << std::endl;
        return 1;
    }
}

