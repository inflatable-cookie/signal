# Fetch Catch2
include(FetchContent)
FetchContent_Declare(
    Catch2
    GIT_REPOSITORY https://github.com/catchorg/Catch2.git
    GIT_TAG v3.5.2
)
FetchContent_MakeAvailable(Catch2)

add_executable(signal-tests
    test_envelope.cpp
    test_router.cpp
    test_ipc_envelope_codec.cpp
    test_audio_bus.cpp
    test_schedule_transport_thread_safety.cpp
    test_graph_runtime.cpp
    test_graph_runtime_phase3.cpp
    test_plugin_hosting.cpp
    test_automation.cpp
    test_recording.cpp
)

target_link_libraries(signal-tests PRIVATE
    signal-core
    Catch2::Catch2WithMain
)

target_include_directories(signal-tests PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../src
)

add_test(NAME SignalTests COMMAND signal-tests)

