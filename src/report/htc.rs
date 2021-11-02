use crate::{cfd, MonitorsLoader};
use rayon::prelude::*;
use std::{error::Error, fs::File, io::Write, path::Path};
use strum::IntoEnumIterator;

/// Chapter section
pub fn chapter_section(
    cfd_case: cfd::CfdCase<{ super::CFD_YEAR }>,
) -> Result<String, Box<dyn Error>> {
    let path_to_case =
        cfd::Baseline::<{ super::CFD_YEAR }>::path().join(
            &<cfd::CfdCase<{ super::CFD_YEAR }> as std::string::ToString>::to_string(&cfd_case),
        );
    let monitors = MonitorsLoader::<{ super::CFD_YEAR }>::default()
        .data_path(path_to_case)
        .load()?;
    Ok(format!(
        r#"
\section{{{}}}
{}
"#,
        &cfd_case.to_pretty_string(),
        if let Some(data) = monitors.htc_latex_table(400f64) {
            data
        } else {
            String::new()
        }
    ))
}
/// Chapter assembly
pub fn chapter(zenith_angle: cfd::ZenithAngle) -> Result<(), Box<dyn Error>> {
    let report_path = Path::new("report");
    let chapter_filename = match zenith_angle {
        cfd::ZenithAngle::Zero => "part3.chapter1.tex",
        cfd::ZenithAngle::Thirty => "part3.chapter2.tex",
        cfd::ZenithAngle::Sixty => "part3.chapter3.tex",
    };
    let cfd_cases = cfd::Baseline::<{ super::CFD_YEAR }>::at_zenith(zenith_angle.clone())
        .into_iter()
        .collect::<Vec<cfd::CfdCase<{ super::CFD_YEAR }>>>();
    let results: Vec<_> = cfd_cases
        .into_par_iter()
        .map(|cfd_case| chapter_section(cfd_case).unwrap())
        .collect();
    let mut file = File::create(report_path.join(chapter_filename))?;
    write!(
        file,
        r#"
\chapter{{{}}}
{}
"#,
        zenith_angle.chapter_title(),
        results.join("\n")
    )?;
    Ok(())
}
/// Part assembly
pub fn part() -> Result<(), Box<dyn Error>> {
    println!(" -->> HTC ...");
    for zenith_angle in cfd::ZenithAngle::iter() {
        chapter(zenith_angle)?;
    }
    Ok(())
}
