add_executable(loophole-signal
    main.cpp
    core/SignalApp.cpp
    core/EngineHost.cpp
    ipc/Envelope.cpp
    ipc/Router.cpp
    domains/EngineDomain.cpp
    domains/TransportDomain.cpp
)

target_include_directories(loophole-signal PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

