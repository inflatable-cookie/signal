# Create a library for shared code
add_library(signal-core STATIC
    core/SignalApp.cpp
    core/EngineHost.cpp
    core/EngineSelfTest.cpp
    core/MeteringService.cpp
    core/ChannelMixService.cpp
    core/AutomationService.cpp
    core/StreamScheduler.cpp
    core/ScheduleData.cpp
    core/GraphNode.cpp
    core/GraphSnapshot.cpp
    core/GraphEngine.cpp
    core/RecordingCapture.cpp
    core/PluginHost.cpp
    core/PluginCrashTracking.cpp
    core/AudioAssetSource.cpp
    clap/ClapPluginLibrary.cpp
    clap/ClapRegistry.cpp
    clap/ClapPluginInstance.cpp
    backend/MiniaudioBackend.cpp
    ipc/IpcEnvelope.cpp
    ipc/IpcEnvelopeCodec.cpp
    ipc/TcpClientSession.cpp
    ipc/TcpServer.cpp
    ipc/DomainDispatcher.cpp
    domains/EngineDomain.cpp
    domains/TransportDomain.cpp
    domains/MeteringDomain.cpp
    domains/MixerDomain.cpp
    domains/AutomationDomain.cpp
    domains/AssetsDomain.cpp
    domains/HardwareDomain.cpp
    domains/NodeDomain.cpp
    logging/Logging.cpp
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

# Add audio debug compile definition if enabled
if(LOOPHOLE_ENABLE_AUDIO_DEBUG)
    target_compile_definitions(signal-core PUBLIC LOOPHOLE_ENABLE_AUDIO_DEBUG)
endif()

# Add official CLAP SDK include directory
target_include_directories(signal-core PUBLIC ${clap_SOURCE_DIR}/include)

# Add miniaudio include directory
target_include_directories(signal-core PUBLIC ${miniaudio_SOURCE_DIR})

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
