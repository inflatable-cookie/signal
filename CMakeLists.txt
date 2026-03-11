cmake_minimum_required(VERSION 3.20)

project(signal LANGUAGES CXX)

# The root CMake surface now exists primarily as a compatibility wrapper around
# the legacy C++ implementation. Active implementation work is expected to land
# in the Rust workspace under `crates/`.
add_subdirectory(legacy/cpp)

