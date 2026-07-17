use std::{fs, path::Path};

use super::{evidence, PhaseFieldClassEvidence, PhaseFieldDirection, Run};

pub(super) fn write(
    root: &Path,
    run: &Run,
    repeated: bool,
    classes: [PhaseFieldClassEvidence; 3],
    direction: PhaseFieldDirection,
) {
    let mut report = format!(
        "rows\t{}\nrepeated\t{repeated}\nevidence_hash\t{:016x}\ndirection\t{direction:?}\nratio\tcontrol\tclass\tcoefficients\tphase_delta_rms\tphase_delta_max\trelation_bins\trelation_before_rms\trelation_after_rms\trelation_before_max\trelation_after_max\n",
        run.rows, run.evidence_hash
    );
    write_classes(&mut report, "all", "all", &classes);
    for group in &run.groups {
        write_classes(
            &mut report,
            &format!("{:.2}", group.ratio),
            group.control,
            &evidence(&group.classes),
        );
    }
    fs::write(root.join("phase-field-attribution.tsv"), report)
        .expect("write phase-field attribution report");
}

fn write_classes(
    report: &mut String,
    ratio: &str,
    control: &str,
    classes: &[PhaseFieldClassEvidence; 3],
) {
    for (name, class) in ["anchor", "interior", "boundary"].into_iter().zip(classes) {
        report.push_str(&format!(
            "{ratio}\t{control}\t{name}\t{}\t{:.12e}\t{:.12e}\t{}\t{:.12e}\t{:.12e}\t{:.12e}\t{:.12e}\n",
            class.coefficients,
            class.phase_delta_rms,
            class.maximum_phase_delta,
            class.relation_bins,
            class.relation_before_rms,
            class.relation_after_rms,
            class.maximum_relation_before,
            class.maximum_relation_after,
        ));
    }
}
