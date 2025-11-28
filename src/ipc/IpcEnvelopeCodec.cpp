#include "ipc/IpcEnvelopeCodec.hpp"
#include "logging/Logging.hpp"
#include <nlohmann/json.hpp>
#include <sstream>

namespace loophole::signal::ipc {

std::string serialiseEnvelope(const IpcEnvelope& env) {
    nlohmann::json j;
    j["v"] = env.version;
    j["id"] = env.id;

    if (env.correlationId.has_value()) {
        j["cid"] = env.correlationId.value();
    } else {
        j["cid"] = nullptr;
    }

    j["ts"] = env.timestamp;
    j["origin"] = originToString(env.origin);
    j["target"] = targetToString(env.target);
    j["domain"] = env.domain;
    j["kind"] = kindToString(env.kind);
    j["name"] = env.name;
    j["priority"] = priorityToString(env.priority);
    j["payload"] = env.payload;

    if (env.error.has_value()) {
        nlohmann::json error_obj;
        error_obj["code"] = env.error->code;
        error_obj["message"] = env.error->message;
        if (!env.error->details.is_null()) {
            error_obj["details"] = env.error->details;
        }
        j["error"] = error_obj;
    } else {
        j["error"] = nullptr;
    }

    return j.dump();
}

std::optional<IpcEnvelope> deserialiseEnvelope(std::string_view line) {
    try {
        nlohmann::json j = nlohmann::json::parse(line);

        IpcEnvelope env;

        // Required fields
        if (!j.contains("v") || !j["v"].is_number_integer()) {
            LOG_VERBOSE({"IpcEnvelopeCodec"}, "Missing or invalid 'v' field");
            return std::nullopt;
        }
        env.version = j["v"].get<int>();

        if (!j.contains("id") || !j["id"].is_string()) {
            LOG_VERBOSE({"IpcEnvelopeCodec"}, "Missing or invalid 'id' field");
            return std::nullopt;
        }
        env.id = j["id"].get<std::string>();

        if (!j.contains("ts") || !j["ts"].is_string()) {
            LOG_VERBOSE({"IpcEnvelopeCodec"}, "Missing or invalid 'ts' field");
            return std::nullopt;
        }
        env.timestamp = j["ts"].get<std::string>();
        if (env.timestamp.empty()) {
            LOG_VERBOSE({"IpcEnvelopeCodec"}, "Empty 'ts' field");
            return std::nullopt;
        }

        if (!j.contains("origin") || !j["origin"].is_string()) {
            LOG_VERBOSE({"IpcEnvelopeCodec"}, "Missing or invalid 'origin' field");
            return std::nullopt;
        }
        auto origin_opt = originFromString(j["origin"].get<std::string>());
        if (!origin_opt.has_value()) {
            std::ostringstream msg;
            msg << "Invalid 'origin' value: " << j["origin"];
            LOG_VERBOSE({"IpcEnvelopeCodec"}, msg.str());
            return std::nullopt;
        }
        env.origin = origin_opt.value();

        if (!j.contains("target") || !j["target"].is_string()) {
            LOG_VERBOSE({"IpcEnvelopeCodec"}, "Missing or invalid 'target' field");
            return std::nullopt;
        }
        auto target_opt = targetFromString(j["target"].get<std::string>());
        if (!target_opt.has_value()) {
            std::ostringstream msg;
            msg << "Invalid 'target' value: " << j["target"];
            LOG_VERBOSE({"IpcEnvelopeCodec"}, msg.str());
            return std::nullopt;
        }
        env.target = target_opt.value();

        if (!j.contains("domain") || !j["domain"].is_string()) {
            LOG_VERBOSE({"IpcEnvelopeCodec"}, "Missing or invalid 'domain' field");
            return std::nullopt;
        }
        env.domain = j["domain"].get<std::string>();

        if (!j.contains("kind") || !j["kind"].is_string()) {
            LOG_VERBOSE({"IpcEnvelopeCodec"}, "Missing or invalid 'kind' field");
            return std::nullopt;
        }
        auto kind_opt = kindFromString(j["kind"].get<std::string>());
        if (!kind_opt.has_value()) {
            std::ostringstream msg;
            msg << "Invalid 'kind' value: " << j["kind"];
            LOG_VERBOSE({"IpcEnvelopeCodec"}, msg.str());
            return std::nullopt;
        }
        env.kind = kind_opt.value();

        if (!j.contains("name") || !j["name"].is_string()) {
            LOG_VERBOSE({"IpcEnvelopeCodec"}, "Missing or invalid 'name' field");
            return std::nullopt;
        }
        env.name = j["name"].get<std::string>();

        if (!j.contains("priority") || !j["priority"].is_string()) {
            LOG_VERBOSE({"IpcEnvelopeCodec"}, "Missing or invalid 'priority' field");
            return std::nullopt;
        }
        auto priority_opt = priorityFromString(j["priority"].get<std::string>());
        if (!priority_opt.has_value()) {
            std::ostringstream msg;
            msg << "Invalid 'priority' value: " << j["priority"];
            LOG_VERBOSE({"IpcEnvelopeCodec"}, msg.str());
            return std::nullopt;
        }
        env.priority = priority_opt.value();

        if (!j.contains("payload") || !j["payload"].is_object()) {
            LOG_VERBOSE({"IpcEnvelopeCodec"}, "Missing or invalid 'payload' field");
            return std::nullopt;
        }
        env.payload = j["payload"];

        // Optional fields
        if (j.contains("cid") && !j["cid"].is_null()) {
            if (j["cid"].is_string()) {
                env.correlationId = j["cid"].get<std::string>();
            }
        }

        if (j.contains("error") && !j["error"].is_null() && j["error"].is_object()) {
            IpcErrorInfo error_info;
            if (j["error"].contains("code") && j["error"]["code"].is_string()) {
                error_info.code = j["error"]["code"].get<std::string>();
            }
            if (j["error"].contains("message") && j["error"]["message"].is_string()) {
                error_info.message = j["error"]["message"].get<std::string>();
            }
            if (j["error"].contains("details")) {
                error_info.details = j["error"]["details"];
            } else {
                error_info.details = nlohmann::json::object();
            }
            env.error = std::make_optional(error_info);
        }

        return env;
    } catch (const nlohmann::json::parse_error& e) {
        LOG_ERROR({"IpcEnvelopeCodec"}, std::string("JSON parse error: ") + e.what());
        return std::nullopt;
    } catch (const nlohmann::json::type_error& e) {
        LOG_ERROR({"IpcEnvelopeCodec"}, std::string("JSON type error: ") + e.what());
        return std::nullopt;
    } catch (const std::exception& e) {
        LOG_ERROR({"IpcEnvelopeCodec"}, std::string("Unexpected error: ") + e.what());
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc

