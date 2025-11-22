#include "ipc/IpcEnvelopeCodec.hpp"
#include <iostream>
#include <nlohmann/json.hpp>

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
            std::cerr << "[IpcEnvelopeCodec] Missing or invalid 'v' field" << std::endl;
            return std::nullopt;
        }
        env.version = j["v"].get<int>();

        if (!j.contains("id") || !j["id"].is_string()) {
            std::cerr << "[IpcEnvelopeCodec] Missing or invalid 'id' field" << std::endl;
            return std::nullopt;
        }
        env.id = j["id"].get<std::string>();

        if (!j.contains("ts") || !j["ts"].is_string()) {
            std::cerr << "[IpcEnvelopeCodec] Missing or invalid 'ts' field" << std::endl;
            return std::nullopt;
        }
        env.timestamp = j["ts"].get<std::string>();
        if (env.timestamp.empty()) {
            std::cerr << "[IpcEnvelopeCodec] Empty 'ts' field" << std::endl;
            return std::nullopt;
        }

        if (!j.contains("origin") || !j["origin"].is_string()) {
            std::cerr << "[IpcEnvelopeCodec] Missing or invalid 'origin' field" << std::endl;
            return std::nullopt;
        }
        auto origin_opt = originFromString(j["origin"].get<std::string>());
        if (!origin_opt.has_value()) {
            std::cerr << "[IpcEnvelopeCodec] Invalid 'origin' value: " << j["origin"] << std::endl;
            return std::nullopt;
        }
        env.origin = origin_opt.value();

        if (!j.contains("target") || !j["target"].is_string()) {
            std::cerr << "[IpcEnvelopeCodec] Missing or invalid 'target' field" << std::endl;
            return std::nullopt;
        }
        auto target_opt = targetFromString(j["target"].get<std::string>());
        if (!target_opt.has_value()) {
            std::cerr << "[IpcEnvelopeCodec] Invalid 'target' value: " << j["target"] << std::endl;
            return std::nullopt;
        }
        env.target = target_opt.value();

        if (!j.contains("domain") || !j["domain"].is_string()) {
            std::cerr << "[IpcEnvelopeCodec] Missing or invalid 'domain' field" << std::endl;
            return std::nullopt;
        }
        env.domain = j["domain"].get<std::string>();

        if (!j.contains("kind") || !j["kind"].is_string()) {
            std::cerr << "[IpcEnvelopeCodec] Missing or invalid 'kind' field" << std::endl;
            return std::nullopt;
        }
        auto kind_opt = kindFromString(j["kind"].get<std::string>());
        if (!kind_opt.has_value()) {
            std::cerr << "[IpcEnvelopeCodec] Invalid 'kind' value: " << j["kind"] << std::endl;
            return std::nullopt;
        }
        env.kind = kind_opt.value();

        if (!j.contains("name") || !j["name"].is_string()) {
            std::cerr << "[IpcEnvelopeCodec] Missing or invalid 'name' field" << std::endl;
            return std::nullopt;
        }
        env.name = j["name"].get<std::string>();

        if (!j.contains("priority") || !j["priority"].is_string()) {
            std::cerr << "[IpcEnvelopeCodec] Missing or invalid 'priority' field" << std::endl;
            return std::nullopt;
        }
        auto priority_opt = priorityFromString(j["priority"].get<std::string>());
        if (!priority_opt.has_value()) {
            std::cerr << "[IpcEnvelopeCodec] Invalid 'priority' value: " << j["priority"] << std::endl;
            return std::nullopt;
        }
        env.priority = priority_opt.value();

        if (!j.contains("payload") || !j["payload"].is_object()) {
            std::cerr << "[IpcEnvelopeCodec] Missing or invalid 'payload' field" << std::endl;
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
        std::cerr << "[IpcEnvelopeCodec] JSON parse error: " << e.what() << std::endl;
        return std::nullopt;
    } catch (const nlohmann::json::type_error& e) {
        std::cerr << "[IpcEnvelopeCodec] JSON type error: " << e.what() << std::endl;
        return std::nullopt;
    } catch (const std::exception& e) {
        std::cerr << "[IpcEnvelopeCodec] Unexpected error: " << e.what() << std::endl;
        return std::nullopt;
    }
}

} // namespace loophole::signal::ipc

