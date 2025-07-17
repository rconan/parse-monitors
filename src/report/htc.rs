use crate::{cfd, cfd::BaselineTrait, MonitorsLoader};
use rayon::prelude::*;
use std::{error::Error, fs::File, io::Write, path::Path};

pub struct HTC<const CFD_YEAR: u32> {
    part: u8,
    stats_time_range: f64,
}
impl<const CFD_YEAR: u32> HTC<CFD_YEAR> {
    pub fn new(part: u8, stats_time_range: f64) -> Self {
        Self {
            part,
            stats_time_range,
        }
    }
}
impl<const CFD_YEAR: u32> super::Report<CFD_YEAR> for HTC<CFD_YEAR> {
    /// Chapter section
    fn chapter_section(
        &self,
        cfd_case: cfd::CfdCase<CFD_YEAR>,
        _: Option<usize>,
    ) -> Result<String, Box<dyn Error>> {
        let path_to_case = cfd::Baseline::<CFD_YEAR>::path().join(&cfd_case.to_string());
        let monitors = MonitorsLoader::<CFD_YEAR>::default()
            .data_path(path_to_case)
            .load()?;
        Ok(format!(
            r#"
\section{{{}}}
\begin{{longtable}}{{crrrr}}\toprule
 ELEMENT & MEAN & STD & MIN & MAX \\\hline
{}
\bottomrule
\end{{longtable}}
"#,
            &cfd_case.to_pretty_string(),
            monitors
                .htc_latex_table(self.stats_time_range)
                .unwrap_or_default()
        ))
    }
    /// Chapter assembly
    fn chapter(
        &self,
        zenith_angle: cfd::ZenithAngle,
        cfd_cases_subset: Option<&[cfd::CfdCase<CFD_YEAR>]>,
    ) -> Result<(), Box<dyn Error>> {
        let report_path = Path::new("report");
        let part = format!("part{}.", self.part);
        let chapter_filename = match zenith_angle {
            cfd::ZenithAngle::Zero => part + "chapter1.tex",
            cfd::ZenithAngle::Thirty => part + "chapter2.tex",
            cfd::ZenithAngle::Sixty => part + "chapter3.tex",
        };
        let cfd_cases = cfd::Baseline::<CFD_YEAR>::at_zenith(zenith_angle)
            .into_iter()
            .filter(|cfd_case| {
                if let Some(cases) = cfd_cases_subset {
                    cases.contains(cfd_case)
                } else {
                    true
                }
            })
            .collect::<Vec<cfd::CfdCase<CFD_YEAR>>>();
        let results: Vec<_> = cfd_cases
            .into_par_iter()
            .map(|cfd_case| self.chapter_section(cfd_case, None).unwrap())
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
    fn part_name(&self) -> String {
        String::from("HTC")
    }
}
