use crate::{cfd, MonitorsLoader};
use rayon::prelude::*;
use std::{error::Error, fs::File, io::Write, path::Path};

pub struct HTC {
    part: u8,
    stats_time_range: f64,
}
impl HTC {
    pub fn new(part: u8, stats_time_range: f64) -> Self {
        Self {
            part,
            stats_time_range,
        }
    }
}
impl super::Report<2021> for HTC {
    /// Chapter section
    fn chapter_section(&self, cfd_case: cfd::CfdCase<2021>) -> Result<String, Box<dyn Error>> {
        let path_to_case = cfd::Baseline::<2021>::path().join(&cfd_case.to_string());
        let monitors = MonitorsLoader::<2021>::default()
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
    fn chapter(&self, zenith_angle: cfd::ZenithAngle) -> Result<(), Box<dyn Error>> {
        let report_path = Path::new("report");
        let part = format!("part{}.", self.part);
        let chapter_filename = match zenith_angle {
            cfd::ZenithAngle::Zero => part + "chapter1.tex",
            cfd::ZenithAngle::Thirty => part + "chapter2.tex",
            cfd::ZenithAngle::Sixty => part + "chapter3.tex",
        };
        let cfd_cases = cfd::Baseline::<2021>::at_zenith(zenith_angle.clone())
            .into_iter()
            .collect::<Vec<cfd::CfdCase<2021>>>();
        let results: Vec<_> = cfd_cases
            .into_par_iter()
            .map(|cfd_case| self.chapter_section(cfd_case).unwrap())
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
