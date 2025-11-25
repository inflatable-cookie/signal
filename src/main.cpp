#include "core/SignalApp.hpp"
#include <iostream>
#include <exception>

int main() {
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

