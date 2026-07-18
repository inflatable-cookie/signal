use std::{fs, path::Path};

use super::{specimen::Specimen, LinkedSubbandFeasibilityDirection, Run};

#[allow(clippy::too_many_arguments)]
pub(super) fn write(
    root: &Path,
    specimen: &Specimen,
    run: &Run,
    repeated: bool,
    stereo_failures: usize,
    local_consistency_failures: usize,
    direction: LinkedSubbandFeasibilityDirection,
) {
    let mut stereo = String::from("ratio\tframes\tphase\tbin_aligned\tcontrol\tscope\tcurrent_ipd\tcandidate_ipd\tcurrent_mid_side\tcandidate_mid_side\tcurrent_correlation\tcandidate_correlation\tcurrent_relation\tcandidate_relation\tstructural_failures\tlocal_improved\tlocal_before\tlocal_after\toutput_hash\n");
    for row in &run.stereo {
        for scope in 0..2 {
            let current = row.current[scope];
            let candidate = row.candidate[scope];
            stereo.push_str(&format!(
                "{:.2}\t{}\t{:.2}\t{}\t{}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\t{}\t{}\t{:.12e}\t{:.12e}\t{:016x}\n",
                row.ratio,
                row.source_frames,
                row.phase,
                row.bin_aligned,
                row.control,
                ["whole", "interior"][scope],
                current.ipd_error_radians,
                candidate.ipd_error_radians,
                current.mid_side_delta_db,
                candidate.mid_side_delta_db,
                current.correlation_delta,
                candidate.correlation_delta,
                current.relation_residual,
                candidate.relation_residual,
                row.structural_failures,
                row.local_windows_improved,
                row.maximum_local_residuals[0],
                row.maximum_local_residuals[1],
                row.output_hash,
            ));
        }
    }
    fs::write(root.join("stereo.tsv"), stereo).expect("write SBSMS stereo report");
    fs::write(root.join("mono.tsv"), &run.mono_report).expect("write SBSMS mono report");
    fs::write(
        root.join("source-trace.tsv"),
        "stage\tpinned_source\tboundary\nmatch\tsrc/sms.cpp:309-360\tadjust2 establishes mutual dupStereo track points\ntrajectory\tsrc/track.cpp:95-164\tupdateFPH derives paired frequency history and analysis-relative synthesis phase\nphase_commit\tsrc/sms.cpp:367-435\tadjust1 postpones paired phase until the counterpart is available\nsynthesis\tsrc/track.cpp:240\tTrack::synth emits direct oscillator samples\nchannel_render\tsrc/sms.cpp:1789\tthe SynthRenderer invokes direct track synthesis per channel\nsum\tsrc/subband.cpp:759-773\trecursive subband output enters one mixer and read timeline\n",
    )
    .expect("write SBSMS source trace");
    let summary = format!(
        "revision\t{}\nsource\t{}\nrepeated\t{}\nstereo_rows\t{}\nstereo_failures\t{}\nlocal_consistency_failures\t{}\nmechanics_errors\t{:.12e},{:.12e},{:.12e},{:.12e},{:.12e},{:.12e}\nmono_hard_failures\t{}\nmono_row_complete_regressions\t{}\nmetrics_worse_than_both_controls\t{}\nrenders\t{}\nsource_frames\t{}\noutput_frames\t{}\nsynthesis_frame_callbacks\t{}\ntime_groups\t{}\ntrack_visits\t{}\ntrack_births\t{}\ntrack_deaths\t{}\nmaximum_tracks_per_time\t{}\nmaximum_track_visits_per_output_read\t{}\nmaximum_peak_rss_bytes\t{}\nelapsed_seconds\t{:.6}\nevidence_hash\t{:016x}\ndirection\t{:?}\n",
        specimen.revision,
        specimen.source.display(),
        repeated,
        run.stereo.len(),
        stereo_failures,
        local_consistency_failures,
        run.mechanics_errors[0],
        run.mechanics_errors[1],
        run.mechanics_errors[2],
        run.mechanics_errors[3],
        run.mechanics_errors[4],
        run.mechanics_errors[5],
        run.mono_hard_failures,
        run.mono_row_complete_regressions,
        run.metrics_worse_than_both_controls,
        run.resources.renders,
        run.resources.source_frames,
        run.resources.output_frames,
        run.resources.synthesis_frames,
        run.resources.time_groups,
        run.resources.track_visits,
        run.resources.track_births,
        run.resources.track_deaths,
        run.resources.maximum_tracks_per_time,
        run.resources.maximum_track_visits_per_output_read,
        run.resources.maximum_peak_rss_bytes,
        run.resources.elapsed_seconds,
        run.evidence_hash,
        direction,
    );
    fs::write(root.join("feasibility.tsv"), summary).expect("write SBSMS feasibility summary");
}
