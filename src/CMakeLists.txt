# Create a library for shared code
add_library(signal-core STATIC
    core/SignalApp.cpp
    core/EngineHost.cpp
    ipc/Envelope.cpp
    ipc/Router.cpp
    domains/EngineDomain.cpp
    domains/TransportDomain.cpp
)

target_include_directories(signal-core PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# Main executable
add_executable(loophole-signal
    main.cpp
)

target_link_libraries(loophole-signal PRIVATE
    signal-core
)

