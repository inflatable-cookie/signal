#[path = "tempo_continuity_arc_surface/arc_cross_assertions.rs"]
mod arc_cross_assertions;
#[path = "tempo_continuity_arc_surface/arc_primary_assertions.rs"]
mod arc_primary_assertions;
#[path = "tempo_continuity_arc_surface/arc_secondary_assertions.rs"]
mod arc_secondary_assertions;
#[path = "tempo_continuity_arc_surface/causes_assertions.rs"]
mod causes_assertions;
#[path = "tempo_continuity_arc_surface/fixtures.rs"]
mod fixtures;

#[test]
fn tempo_continuity_calibrates_causes_and_unresolved_spans() {
    let cases = fixtures::arc_surface_cases();
    causes_assertions::assert_causes_and_unresolved_spans(&cases);
}

#[test]
fn tempo_continuity_calibrates_arcs_and_arc_support() {
    let cases = fixtures::arc_surface_cases();
    arc_primary_assertions::assert_integer_and_core_window(&cases);
    arc_secondary_assertions::assert_guarded_refined_and_deferred(&cases);
    arc_cross_assertions::assert_cross_surface_relationships(&cases);
}
