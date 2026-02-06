# Create a library for shared code
add_library(signal-core STATIC
    core/SignalApp.cpp
    core/EngineHost.cpp
    core/EngineSelfTest.cpp
    core/MeteringService.cpp
    core/AutomationService.cpp
    core/StreamScheduler.cpp
    core/ScheduleData.cpp
    core/MidiInputRouter.cpp
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
    vst3/Vst3Registry.cpp
    vst3/Vst3Backend.cpp
    vst3/Vst3PluginInstance.cpp
	    backend/MiniaudioBackend.cpp
	    ipc/IpcEnvelope.cpp
	    ipc/BinaryEnvelopeCodec.cpp
        ipc/binary_envelope/PayloadCodecs.cpp
        ipc/binary_envelope/assets/codecs.cpp
        ipc/binary_envelope/assets/command_register_audio_asset.cpp
        ipc/binary_envelope/automation/codecs.cpp
        ipc/binary_envelope/automation/command_automation_snapshot.cpp
        ipc/binary_envelope/control/codecs.cpp
        ipc/binary_envelope/control/event_control_event.cpp
        ipc/binary_envelope/control/event_device_inventory.cpp
        ipc/binary_envelope/diagnostics/codecs.cpp
        ipc/binary_envelope/diagnostics/event_engine_metrics.cpp
        ipc/binary_envelope/engine/codecs.cpp
        ipc/binary_envelope/engine/command_graph_snapshot.cpp
        ipc/binary_envelope/engine/command_playback_schedule_snapshot.cpp
        ipc/binary_envelope/engine/command_schema_only.cpp
        ipc/binary_envelope/engine/event_state.cpp
        ipc/binary_envelope/engine/event_self_test_result.cpp
        ipc/binary_envelope/hardware/codecs.cpp
        ipc/binary_envelope/hardware/command_refresh_output_devices.cpp
        ipc/binary_envelope/hardware/command_select_output_device.cpp
        ipc/binary_envelope/hardware/event_state.cpp
        ipc/binary_envelope/metering/codecs.cpp
        ipc/binary_envelope/metering/event_update.cpp
        ipc/binary_envelope/node/codecs.cpp
        ipc/binary_envelope/node/command_set_parameter.cpp
        ipc/binary_envelope/transport/codecs.cpp
        ipc/binary_envelope/transport/command_play.cpp
        ipc/binary_envelope/transport/command_stop.cpp
        ipc/binary_envelope/transport/command_seek.cpp
        ipc/binary_envelope/transport/command_set_loop_enabled.cpp
        ipc/binary_envelope/transport/command_set_loop_region.cpp
        ipc/binary_envelope/transport/event_state.cpp
        ipc/binary_envelope/transport/event_position_update.cpp
	    ipc/TcpClientSession.cpp
	    ipc/TcpServer.cpp
    ipc/DomainDispatcher.cpp
    domains/EngineDomain.cpp
    domains/TransportDomain.cpp
    domains/MeteringDomain.cpp
    domains/AutomationDomain.cpp
    domains/AssetsDomain.cpp
    domains/control/MidiControlNormaliser.cpp
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

if(TARGET libremidi::libremidi)
    target_link_libraries(signal-core PUBLIC libremidi::libremidi)
elseif(TARGET libremidi)
    target_link_libraries(signal-core PUBLIC libremidi)
else()
    message(FATAL_ERROR "libremidi target not found")
endif()

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

# Add VST3 SDK include directories and compile-time flags
if(SIGNAL_ENABLE_VST3)
    target_compile_definitions(signal-core PUBLIC SIGNAL_ENABLE_VST3=1)
    if(vst3sdk_POPULATED)
        target_compile_definitions(signal-core PUBLIC SIGNAL_ENABLE_VST3_SDK=1)
        target_include_directories(signal-core PUBLIC
            ${vst3sdk_SOURCE_DIR}
            ${vst3sdk_SOURCE_DIR}/base/source
            ${vst3sdk_SOURCE_DIR}/pluginterfaces
            ${vst3sdk_SOURCE_DIR}/public.sdk/source
        )
    else()
        target_compile_definitions(signal-core PUBLIC SIGNAL_ENABLE_VST3_SDK=0)
    endif()
else()
    target_compile_definitions(signal-core PUBLIC SIGNAL_ENABLE_VST3=0 SIGNAL_ENABLE_VST3_SDK=0)
endif()

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
