# Fetch Catch2
include(FetchContent)
FetchContent_Declare(
    Catch2
    GIT_REPOSITORY https://github.com/catchorg/Catch2.git
    GIT_TAG v3.5.2
)
FetchContent_MakeAvailable(Catch2)

add_executable(signal-tests
    test_domain_dispatcher.cpp
    test_binary_envelope_codec.cpp
    test_midi_control_normaliser.cpp
    test_audio_bus.cpp
    test_schedule_transport_thread_safety.cpp
    test_graph_runtime.cpp
    test_graph_runtime_phase3.cpp
    test_file_audio_asset_source.cpp
    test_plugin_hosting.cpp
    test_node_muted_parameter.cpp
    test_automation.cpp
    test_recording.cpp
    test_playback_path.cpp
)

target_link_libraries(signal-tests PRIVATE
    signal-core
    Catch2::Catch2WithMain
)

target_include_directories(signal-tests PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../src
)

add_test(NAME SignalTests COMMAND signal-tests)
