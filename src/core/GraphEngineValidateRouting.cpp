#include "core/GraphEngine.hpp"
#include "core/GraphNodes.hpp"
#include "core/GraphSnapshotHelpers.hpp"
#include "logging/Logging.hpp"
#include <sstream>

void GraphEngine::validateRouting() {
    // Validate all node-to-node connections according to routing rules
    // Rules:
    // 1. Channel count equality: fromNode.outputChannels == toNode.inputChannels
    // 2. Audio-only nodes cannot connect to MIDI-only nodes (and vice versa)
    // 3. HardwareAudioOutputNode must accept layouts matching hardware (for now, stereo only)
    // 4. MidiLaneNode has 0 audio channels - audio connections are invalid

    int invalidConnections = 0;

    for (auto& conn : _connections) {
        // Skip if already marked invalid
        if (!conn.isValid) {
            continue;
        }

        GraphNode* fromNode = findNode(conn.fromNodeId);
        GraphNode* toNode = findNode(conn.toNodeId);

        if (!fromNode || !toNode) {
            // Node not found - mark as invalid
            conn.isValid = false;
            invalidConnections++;
            std::ostringstream msg;
            msg << "Invalid connection: node not found - "
                << (fromNode ? conn.toNodeId : conn.fromNodeId);
            LOG_ERROR({"GraphEngine", "Routing"}, msg.str());
            continue;
        }

        const auto& fromConfig = fromNode->getAudioConfig();
        const auto& toConfig = toNode->getAudioConfig();

        // Rule 1: Channel count equality for audio connections
        // Exception: HardwareAudioOutputNode can handle mismatches (handled in Rule 3)
        ChannelCompatibility compat = checkChannelCompatibility(*fromNode, *toNode);
        bool isHardwareOutputNode = (toNode->getKind() == NodeKind::HardwareAudioOutput);

        if (compat.isMismatch && !isHardwareOutputNode) {
            // Channel mismatch - mark connection as invalid (unless it's a hardware output node)
            conn.isValid = false;
            invalidConnections++;
            std::ostringstream msg;
            msg << "Channel mismatch in connection: " << conn.fromNodeId
                << " (" << compat.sourceChannels << " ch) -> "
                << conn.toNodeId << " (" << compat.destChannels << " ch)";
            // Add node kind information for better diagnostics
            std::string fromKind = nodeKindToString(fromNode->getKind());
            std::string toKind = nodeKindToString(toNode->getKind());
            msg << " (from " << fromKind << " to " << toKind << ")";
            LOG_ERROR({"GraphEngine", "Routing"}, msg.str());
            continue;
        } else if (compat.isCompatible) {
            // Log info for compatible connections (debug level)
            LOG_DEBUG({"GraphEngine", "Routing"},
                std::string("Compatible connection: ") + conn.fromNodeId +
                " (" + std::to_string(compat.sourceChannels) + " ch) -> " +
                conn.toNodeId + " (" + std::to_string(compat.destChannels) + " ch)");
        }

        // Rule 2: Audio-only nodes cannot connect to MIDI-only nodes
        // (MIDI routing is separate and always valid)
        // This is implicitly handled by channel count check above

        // Rule 3: HardwareAudioOutputNode validation
        // HardwareAudioOutputNode channel count comes from actual hardware device
        // HardwareAudioOutputNode can handle channel mismatches (expansion/truncation), so we log warnings but don't mark as invalid
        if (toNode->getKind() == NodeKind::HardwareAudioOutput) {
            // HardwareAudioOutputNode channel count is set from actual device in prepare()
            // HardwareAudioOutputNode will handle channel expansion/truncation in process()
            if (compat.isMismatch) {
                std::ostringstream msg;
                msg << "HardwareAudioOutputNode " << conn.toNodeId
                    << " channel mismatch: upstream has " << compat.sourceChannels
                    << " ch, device has " << compat.destChannels << " ch";
                if (compat.sourceChannels < compat.destChannels) {
                    msg << " (will expand: duplicate channels)";
                } else {
                    msg << " (will truncate: drop extra channels)";
                }
                LOG_WARN({"GraphEngine", "Routing", "HardwareAudioOutput"}, msg.str());
                // Connection is still valid - HardwareAudioOutputNode handles the mismatch
                conn.isValid = true;
            } else if (compat.isCompatible) {
                LOG_DEBUG({"GraphEngine", "Routing", "HardwareAudioOutput"},
                    std::string("HardwareAudioOutputNode ") + conn.toNodeId + " routing compatible: " +
                    std::to_string(compat.destChannels) + " channels");
            }
        }

        // Rule 4: MidiLaneNode has 0 audio channels
        if (fromNode->getKind() == NodeKind::MidiLane) {
            if (fromConfig.numOutputChannels > 0) {
                // MidiLaneNode should have 0 audio channels
                // This shouldn't happen if config is set correctly, but check anyway
                std::ostringstream msg;
                msg << "MidiLaneNode " << conn.fromNodeId
                    << " has audio channels (" << fromConfig.numOutputChannels
                    << ") - audio connection invalid";
                LOG_ERROR({"GraphEngine", "Routing"}, msg.str());
                conn.isValid = false;
                invalidConnections++;
                continue;
            }
        }

        // Rule 5: Mixer-related node validation (SendNode, ReceiveNode, FaderNode)
        // These nodes participate in routing and must have compatible channel counts
        bool isMixerNode = (
            fromNode->getKind() == NodeKind::Send ||
            toNode->getKind() == NodeKind::Receive ||
            fromNode->getKind() == NodeKind::Fader ||
            toNode->getKind() == NodeKind::Fader
        );

        if (isMixerNode && compat.isCompatible) {
            // Log routing compatibility at debug level
            LOG_DEBUG({"GraphEngine", "Routing", "Mixer"},
                std::string("Mixer routing compatible: ") + conn.fromNodeId +
                " (" + std::to_string(compat.sourceChannels) + " ch) -> " +
                conn.toNodeId + " (" + std::to_string(compat.destChannels) + " ch)");
        }
    }

    // Count compatible connections for summary
    int compatibleConnections = 0;
    for (const auto& conn : _connections) {
        if (conn.isValid) {
            GraphNode* fromNode = findNode(conn.fromNodeId);
            GraphNode* toNode = findNode(conn.toNodeId);
            if (fromNode && toNode) {
                ChannelCompatibility compat = checkChannelCompatibility(*fromNode, *toNode);
                if (compat.isCompatible) {
                    compatibleConnections++;
                }
            }
        }
    }

    // Log summary
    if (invalidConnections > 0) {
        std::ostringstream msg;
        msg << "Routing validation: " << compatibleConnections << " compatible, "
            << invalidConnections << " invalid connection(s) marked and will be skipped at render time";
        LOG_WARN({"GraphEngine", "Routing"}, msg.str());
    } else {
        std::ostringstream msg;
        msg << "Routing validation: " << compatibleConnections << " compatible connection(s)";
        LOG_DEBUG({"GraphEngine", "Routing"}, msg.str());
    }
}
