use crate::{cfd, MonitorsLoader};
use glob::glob;
use rayon::prelude::*;
use std::{error::Error, fs::File, io::Write, path::Path};

pub struct WindLoads {
    part: u8,
    stats_time_range: f64,
}
impl WindLoads {
    pub fn new(part: u8, stats_time_range: f64) -> Self {
        Self {
            part,
            stats_time_range,
        }
    }
}
impl super::Report<2021> for WindLoads {
    /// Chapter section
    fn chapter_section(&self, cfd_case: cfd::CfdCase<2021>) -> Result<String, Box<dyn Error>> {
        let path_to_case = cfd::Baseline::<2021>::path().join(&cfd_case.to_string());
        let pattern = path_to_case
            .join("scenes")
            .join("vort_tel_vort_tel*.png")
            .to_str()
            .unwrap()
            .to_owned();
        let paths = glob(&pattern).expect("Failed to read glob pattern");
        let vort_pic = paths.last().unwrap()?;
        let monitors = MonitorsLoader::<2021>::default()
            .data_path(path_to_case.clone())
            .load()?;
        Ok(format!(
            r#"
\section{{{}}}

\includegraphics[width=0.8\textwidth]{{{:?}}}

\subsection{{Forces [N]}}
{}
\subsubsection{{C-Rings}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{M1 Cell}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{Lower trusses}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{Upper trusses}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{Top-end}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{M2 segments}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{M1 \& M2 baffles}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{M1 outer covers}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{M1 inner covers}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{GIR}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{Prime focus assembly arms}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{Laser launch assemblies}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{Platforms \& cable wraps}}
\includegraphics[width=0.8\textwidth]{{{:?}}}

\subsubsection{{Moments [N.M]}}
{}
"#,
            &cfd_case.to_pretty_string(),
            vort_pic,
            if let Some(data) = monitors.force_latex_table(self.stats_time_range) {
                data
            } else {
                String::new()
            },
            path_to_case.join("c-ring_parts.png"),
            path_to_case.join("m1-cell.png"),
            path_to_case.join("lower-truss.png"),
            path_to_case.join("upper-truss.png"),
            path_to_case.join("top-end.png"),
            path_to_case.join("m2-segments.png"),
            path_to_case.join("m12-baffles.png"),
            path_to_case.join("m1-outer-covers.png"),
            path_to_case.join("m1-inner-covers.png"),
            path_to_case.join("gir.png"),
            path_to_case.join("pfa-arms.png"),
            path_to_case.join("lgs.png"),
            path_to_case.join("platforms-cables.png"),
            if let Some(data) = monitors.moment_latex_table(self.stats_time_range) {
                data
            } else {
                String::new()
            }
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
}
