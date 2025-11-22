# Create a library for shared code
add_library(signal-core STATIC
    core/SignalApp.cpp
    core/EngineHost.cpp
    ipc/Envelope.cpp
    ipc/IpcEnvelope.cpp
    ipc/IpcEnvelopeCodec.cpp
    ipc/Router.cpp
    ipc/TcpClientSession.cpp
    ipc/TcpServer.cpp
    ipc/DomainDispatcher.cpp
    domains/EngineDomain.cpp
    domains/TransportDomain.cpp
)

target_include_directories(signal-core PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_libraries(signal-core PUBLIC
    nlohmann_json::nlohmann_json
)

# Add Asio include directory and define ASIO_STANDALONE
# Standalone Asio doesn't provide a CMake target, so we handle it manually
target_include_directories(signal-core PUBLIC ${asio_SOURCE_DIR}/asio/include)
target_compile_definitions(signal-core PUBLIC ASIO_STANDALONE)

# Link system libraries that Asio needs
if(UNIX AND NOT APPLE)
    target_link_libraries(signal-core PUBLIC pthread)
endif()
if(APPLE)
    find_library(CF_LIBRARY CoreFoundation REQUIRED)
    target_link_libraries(signal-core PUBLIC ${CF_LIBRARY})
endif()

# Main executable
add_executable(loophole-signal
    main.cpp
)

target_link_libraries(loophole-signal PRIVATE
    signal-core
)

