cmake_minimum_required(VERSION 3.20)

project(loophole-signal LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(SIGNAL_BUILD_TESTS "Build Signal tests" ON)
option(LOOPHOLE_ENABLE_AUDIO_DEBUG "Enable audio debug logging and probes" ON)
option(SIGNAL_ENABLE_VST3 "Enable VST3 backend scaffolding" ON)

# Fetch nlohmann/json
include(FetchContent)
FetchContent_Declare(
    nlohmann_json
    GIT_REPOSITORY https://github.com/nlohmann/json.git
    GIT_TAG v3.11.3
)
FetchContent_MakeAvailable(nlohmann_json)

# Fetch standalone Asio (header-only)
FetchContent_Declare(
    asio
    GIT_REPOSITORY https://github.com/chriskohlhoff/asio.git
    GIT_TAG asio-1-28-0
)
FetchContent_MakeAvailable(asio)

# Fetch official CLAP SDK (header-only)
FetchContent_Declare(
    clap
    GIT_REPOSITORY https://github.com/free-audio/clap.git
    GIT_TAG main
)
FetchContent_MakeAvailable(clap)

# Fetch miniaudio (single-header audio library)
FetchContent_Declare(
    miniaudio
    GIT_REPOSITORY https://github.com/mackron/miniaudio.git
    GIT_TAG master
)
FetchContent_MakeAvailable(miniaudio)

# Fetch libremidi (cross-platform MIDI library)
FetchContent_Declare(
    libremidi
    GIT_REPOSITORY https://github.com/celtera/libremidi.git
    GIT_TAG v5.4.3
)
FetchContent_MakeAvailable(libremidi)

if(SIGNAL_ENABLE_VST3)
    # Fetch official Steinberg VST3 SDK (header/source dependency for backend work)
    FetchContent_Declare(
        vst3sdk
        GIT_REPOSITORY https://github.com/steinbergmedia/vst3sdk.git
        GIT_TAG v3.7.14_build_55
        EXCLUDE_FROM_ALL
    )
    FetchContent_MakeAvailable(vst3sdk)
endif()

# Create an interface target for standalone Asio
if(NOT TARGET asio::asio)
    add_library(asio_standalone INTERFACE)
    add_library(asio::asio ALIAS asio_standalone)
    target_include_directories(asio_standalone INTERFACE ${asio_SOURCE_DIR}/asio/include)
    target_compile_definitions(asio_standalone INTERFACE ASIO_STANDALONE)
    # Link with system libraries that Asio needs
    if(UNIX AND NOT APPLE)
        target_link_libraries(asio_standalone INTERFACE pthread)
    endif()
    if(APPLE)
        # macOS needs system libraries for networking
        find_library(CF_LIBRARY CoreFoundation REQUIRED)
        target_link_libraries(asio_standalone INTERFACE ${CF_LIBRARY})
    endif()
endif()

add_subdirectory(src)

if(SIGNAL_BUILD_TESTS)
    enable_testing()
    add_subdirectory(tests)
endif()
