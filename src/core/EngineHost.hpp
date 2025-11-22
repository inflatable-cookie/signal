#pragma once

class EngineHost {
public:
    enum class State {
        Stopped,
        Running
    };

    EngineHost();
    ~EngineHost();

    void start();
    void stop();
    void reset();

    State state() const noexcept;

private:
    State _state;
};

