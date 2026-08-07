use super::super::support::*;
use super::super::*;

#[test]
fn compile_rejects_events_without_processor_and_unsorted_events() {
    let buffer = event_buffer(vec![RenderPluginEvent {
        frame: 0,
        channel: 0,
        kind: RenderPluginEventKind::NoteOff { key: 0 },
    }]);
    let mut spec = processor_spec(None);
    spec.stages[1].events = Some(buffer);
    let (mut controller, _executor) = render_plane();
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(
        error.message.contains("without a plugin processor"),
        "{error}"
    );

    let handle = RenderPluginProcessor::new(RecordingEventProcessor::new() as Arc<_>);
    let unsorted = event_buffer(vec![
        RenderPluginEvent {
            frame: 10,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 0 },
        },
        RenderPluginEvent {
            frame: 5,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 0 },
        },
    ]);
    let spec = events_spec(handle, unsorted);
    let (mut controller, _executor) = render_plane();
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(error.message.contains("not sorted by frame"), "{error}");
}

#[test]
fn event_buffer_swap_is_structural_not_a_gain_fast_path() {
    let handle = RenderPluginProcessor::new(RecordingEventProcessor::new() as Arc<_>);
    let event = RenderPluginEvent {
        frame: 0,
        channel: 0,
        kind: RenderPluginEventKind::NoteOff { key: 0 },
    };
    let with_a = events_spec(handle, event_buffer(vec![event]));
    // Clone shares the Arc: gain-only diff logic sees no change.
    let with_a_again = with_a.clone();
    assert_eq!(with_a.differs_only_in_gains(&with_a_again), Some(vec![]));
    // A rebuilt buffer (same content, new Arc) is structural.
    let mut with_b = with_a.clone();
    with_b.stages[1].events = Some(event_buffer(vec![event]));
    assert_eq!(with_a.differs_only_in_gains(&with_b), None);
}
