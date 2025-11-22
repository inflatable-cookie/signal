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
)

target_link_libraries(signal-tests PRIVATE
    signal-core
    Catch2::Catch2WithMain
)

target_include_directories(signal-tests PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../src
)

add_test(NAME SignalTests COMMAND signal-tests)

