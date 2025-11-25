// Minimal CLAP SDK header definitions for Phase 5
// This is a simplified version of the CLAP SDK for integration
// In production, use the official CLAP SDK from https://github.com/free-audio/clap

#pragma once

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

#define CLAP_VERSION_MAJOR 1
#define CLAP_VERSION_MINOR 0
#define CLAP_VERSION_REVISION 0
#define CLAP_VERSION ((CLAP_VERSION_MAJOR << 16) | (CLAP_VERSION_MINOR << 8) | CLAP_VERSION_REVISION)
#define CLAP_VERSION_STRING "1.0.0"

// Type definitions (must be before structs that use them)
typedef uint32_t clap_id;
typedef uint32_t clap_version_t;

// CLAP process status (must be before structs that use it)
enum clap_process_status {
    CLAP_PROCESS_ERROR = 0,
    CLAP_PROCESS_CONTINUE = 1,
    CLAP_PROCESS_TAIL = 2,
    CLAP_PROCESS_SLEEP = 3,
};

// CLAP plugin entry
struct clap_plugin_entry {
    uint32_t clap_version;
    void (*init)(const char* path);
    void (*deinit)(void);
    const void* (*get_factory)(const char* factory_id);
};

// CLAP plugin factory
struct clap_plugin_factory {
    uint32_t (*get_plugin_count)(const struct clap_plugin_factory* factory);
    const struct clap_plugin_descriptor* (*get_plugin_descriptor)(
        const struct clap_plugin_factory* factory,
        uint32_t index
    );
    const struct clap_plugin* (*create_plugin)(
        const struct clap_plugin_factory* factory,
        const struct clap_host* host,
        const char* plugin_id
    );
};

// CLAP plugin descriptor
struct clap_plugin_descriptor {
    const char* clap_version;
    const char* id;
    const char* name;
    const char* vendor;
    const char* url;
    const char* manual_url;
    const char* support_url;
    const char* version;
    const char* description;
    const char* features[16]; // Null-terminated array
};

// CLAP host
struct clap_host {
    void* host_data;
    const char* clap_version;
    void (*request_restart)(const struct clap_host* host);
    void (*request_process)(const struct clap_host* host);
    void (*request_callback)(const struct clap_host* host);
    const void* (*get_extension)(const struct clap_host* host, const char* extension_id);
};

// CLAP plugin
struct clap_plugin {
    const struct clap_plugin_descriptor* desc;
    void* plugin_data;
    bool (*init)(const struct clap_plugin* plugin);
    void (*destroy)(const struct clap_plugin* plugin);
    bool (*activate)(
        const struct clap_plugin* plugin,
        double sample_rate,
        uint32_t min_frames_count,
        uint32_t max_frames_count
    );
    void (*deactivate)(const struct clap_plugin* plugin);
    bool (*start_processing)(const struct clap_plugin* plugin);
    void (*stop_processing)(const struct clap_plugin* plugin);
    void (*reset)(const struct clap_plugin* plugin);
    clap_process_status (*process)(
        const struct clap_plugin* plugin,
        const struct clap_process* process
    );
    const void* (*get_extension)(const struct clap_plugin* plugin, const char* id);
    void (*on_main_thread)(const struct clap_plugin* plugin);
};

// CLAP process
struct clap_process {
    uint32_t steady_time;
    uint32_t frames_count;
    struct clap_audio_buffer* audio_inputs;
    uint32_t audio_inputs_count;
    struct clap_audio_buffer* audio_outputs;
    uint32_t audio_outputs_count;
    const struct clap_input_events* in_events;
    struct clap_output_events* out_events;
};

// CLAP audio buffer
struct clap_audio_buffer {
    uint32_t channel_count;
    uint32_t latency;
    bool constant_mask;
    const float* const* data32; // Non-interleaved float32
    double* const* data64;      // Non-interleaved float64 (optional)
};

// CLAP input events
struct clap_input_events {
    void* ctx; // Context pointer for host use
    uint32_t (*size)(const struct clap_input_events* events);
    const struct clap_event_header* (*get)(const struct clap_input_events* events, uint32_t index);
};

// CLAP output events
struct clap_output_events {
    void* ctx; // Context pointer for host use
    bool (*push)(const struct clap_output_events* events, const struct clap_event_header* event);
    bool (*try_push)(const struct clap_output_events* events, const struct clap_event_header* event);
};

// CLAP event header
struct clap_event_header {
    uint16_t size;
    uint16_t space_id;
    uint32_t type;
    uint32_t flags;
    uint32_t time;
};

// CLAP event types
#define CLAP_EVENT_NOTE_ON 0
#define CLAP_EVENT_NOTE_OFF 1
#define CLAP_EVENT_NOTE_CHOKE 2
#define CLAP_EVENT_NOTE_EXPRESSION 3
#define CLAP_EVENT_PARAM_VALUE 4
#define CLAP_EVENT_PARAM_MOD 5
#define CLAP_EVENT_PARAM_GESTURE_BEGIN 6
#define CLAP_EVENT_PARAM_GESTURE_END 7
#define CLAP_EVENT_TRANSPORT 8
#define CLAP_EVENT_MIDI 9
#define CLAP_EVENT_MIDI_SYSEX 10
#define CLAP_EVENT_MIDI2 11

// CLAP note event
struct clap_event_note {
    struct clap_event_header header;
    int16_t note_id;
    int16_t port_index;
    int16_t channel;
    int16_t key;
    double tuning;
    double velocity;
};

// CLAP MIDI event
struct clap_event_midi {
    struct clap_event_header header;
    uint16_t port_index;
    uint8_t data[3];
};

// CLAP parameter extension ID
#define CLAP_EXT_PARAMS "clap.params"

// CLAP parameter extension
struct clap_plugin_params {
    uint32_t (*count)(const struct clap_plugin* plugin);
    bool (*get_info)(
        const struct clap_plugin* plugin,
        uint32_t param_index,
        struct clap_param_info* param_info
    );
    bool (*get_value)(
        const struct clap_plugin* plugin,
        clap_id param_id,
        double* value
    );
    bool (*value_to_text)(
        const struct clap_plugin* plugin,
        clap_id param_id,
        double value,
        char* display,
        uint32_t size
    );
    bool (*text_to_value)(
        const struct clap_plugin* plugin,
        clap_id param_id,
        const char* display,
        double* value
    );
    void (*flush)(
        const struct clap_plugin* plugin,
        const struct clap_input_events* in,
        const struct clap_output_events* out
    );
};

// CLAP parameter info
struct clap_param_info {
    clap_id id;
    uint32_t flags;
    double min_value;
    double max_value;
    double default_value;
    const char* name;
    const char* module;
};

// CLAP state extension ID
#define CLAP_EXT_STATE "clap.state"

// CLAP state extension
struct clap_plugin_state {
    bool (*save)(const struct clap_plugin* plugin, const struct clap_ostream* stream);
    bool (*load)(const struct clap_plugin* plugin, const struct clap_istream* stream);
};

// CLAP stream interfaces
struct clap_ostream {
    void* ctx;
    int64_t (*write)(const struct clap_ostream* stream, const void* buffer, uint64_t size);
};

struct clap_istream {
    void* ctx;
    int64_t (*read)(const struct clap_istream* stream, void* buffer, uint64_t size);
};

// Factory IDs
#define CLAP_PLUGIN_FACTORY_ID "clap.plugin-factory"

#ifdef __cplusplus
}
#endif

