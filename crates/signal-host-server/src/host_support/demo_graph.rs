use signal_graph::{GraphNodeExecutionClass, GraphStageSpec};
use signal_runtime::{GraphNodeProjection, GraphProjection};

pub(crate) fn server_demo_graph_projection() -> GraphProjection {
    GraphProjection {
        graph_id: "signal.host.server.demo".into(),
        node_count: 3,
        nodes: vec![
            GraphNodeProjection {
                node_id: "input-shape".into(),
                execution_class: GraphNodeExecutionClass::PureTransform,
                latency_samples: 0,
                stages: vec![
                    GraphStageSpec::Gain { linear: 0.6 },
                    GraphStageSpec::Bias { amount: -0.04 },
                ],
            },
            GraphNodeProjection {
                node_id: "drive".into(),
                execution_class: GraphNodeExecutionClass::PluginBacked,
                latency_samples: 0,
                stages: vec![GraphStageSpec::TanhDrive { drive: 1.6 }],
            },
            GraphNodeProjection {
                node_id: "output-trim".into(),
                execution_class: GraphNodeExecutionClass::LatencyBearing,
                latency_samples: 32,
                stages: vec![
                    GraphStageSpec::StereoBalance { balance: 0.3 },
                    GraphStageSpec::HardClip { threshold: 0.7 },
                ],
            },
        ],
    }
}
