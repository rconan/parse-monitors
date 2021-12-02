use crate::{cfd, report::Report, Mirror, MonitorsLoader};
use glob::glob;
use rayon::prelude::*;
use std::{error::Error, fs::File, io::Write, path::Path};

#[derive(Default)]
pub struct WindLoads {
    part: u8,
    stats_time_range: f64,
    xmon: Option<String>,
    detrend: bool,
    last_time_range: Option<usize>,
    show_pressure: bool,
}
impl WindLoads {
    pub fn new(part: u8, stats_time_range: f64) -> Self {
        Self {
            part,
            stats_time_range,
            ..Default::default()
        }
    }
    pub fn exclude_monitors(self, xmon: &str) -> Self {
        Self {
            xmon: Some(xmon.to_string()),
            ..self
        }
    }
    pub fn detrend(self) -> Self {
        Self {
            detrend: true,
            ..self
        }
    }
    pub fn keep_last(self, period: usize) -> Self {
        Self {
            last_time_range: Some(period),
            ..self
        }
    }
    pub fn show_m12_pressure(self) -> Self {
        Self {
            show_pressure: true,
            ..self
        }
    }
}
impl super::Report<2021> for WindLoads {
    /// Chapter section
    fn chapter_section(
        &self,
        cfd_case: cfd::CfdCase<2021>,
        _: Option<usize>,
    ) -> Result<String, Box<dyn Error>> {
        let path_to_case = cfd::Baseline::<2021>::path().join(&cfd_case.to_string());
        let pattern = path_to_case
            .join("scenes")
            .join("vort_tel_vort_tel*.png")
            .to_str()
            .unwrap()
            .to_owned();
        let paths = glob(&pattern).expect("Failed to read glob pattern");
        let vort_pic = paths.last().unwrap()?.with_extension("");
        let mut monitors = if let Some(xmon) = &self.xmon {
            MonitorsLoader::<2021>::default()
                .data_path(path_to_case.clone())
                .exclude_filter(xmon)
                .load()?
        } else {
            MonitorsLoader::<2021>::default()
                .data_path(path_to_case.clone())
                .load()?
        };
        if let Some(period) = self.last_time_range {
            monitors.keep_last(period);
        }
        let parts_suffix = if self.detrend {
            monitors.detrend();
            String::from("-detrend")
        } else {
            String::new()
        };
        if let (Ok(m1), Ok(m1_net)) = (
            Mirror::m1(path_to_case.clone()).load(),
            Mirror::m1(path_to_case.clone()).net_force().load(),
        ) {
            let m1_pressure_map = path_to_case.join("m1_pressure_map.png").with_extension("");
            let m1_pressure_mean = path_to_case
                .join("m1_pressure-stats_mean.png")
                .with_extension("");
            let m1_pressure_std = path_to_case
                .join("m1_pressure-stats_std.png")
                .with_extension("");
            let m2_pressure_map = path_to_case.join("m2_pressure_map.png").with_extension("");
            let m2_pressure_mean = path_to_case
                .join("m2_pressure-stats_mean.png")
                .with_extension("");
            let m2_pressure_std = path_to_case
                .join("m2_pressure-stats_std.png")
                .with_extension("");
            let m12_pressures = if self.show_pressure {
                format!(
                    r#"
\subsection{{Pressure}}
\subsubsection{{M1 pressure snapshot}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M1 segment average}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M1 segment standard deviation}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}

\subsubsection{{M2 pressure snapshot}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M2 segment average}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M2 segment standard deviation}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
"#,
                    m1_pressure_map,
                    m1_pressure_mean,
                    m1_pressure_std,
                    m2_pressure_map,
                    m2_pressure_mean,
                    m2_pressure_std,
                )
            } else {
                String::new()
            };
            Ok(format!(
                r#"
\section{{{}}}
\label{{{}}}

\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}

\subsection{{Forces [N]}}
\begin{{longtable}}{{crrrr}}\toprule
 ELEMENT & MEAN & STD & MIN & MAX \\\hline
{}
\bottomrule
\end{{longtable}}

\subsubsection{{M1 segment net forces}}
\begin{{longtable}}{{crrrr}}\toprule
 ELEMENT & MEAN & STD & MIN & MAX \\\hline
{}
\bottomrule
\end{{longtable}}

\subsubsection{{C-Rings}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M1 Cell}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M1 segments}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{Lower trusses}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{Upper trusses}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{Top-end}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M2 segments}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M1 \& M2 baffles}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M1 outer covers}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M1 inner covers}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{GIR}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{Prime focus assembly arms}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{Laser launch assemblies}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{Platforms \& cable wraps}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}

\subsection{{Moments [N.M]}}
\begin{{longtable}}{{crrrr}}\toprule
 ELEMENT & MEAN & STD & MIN & MAX \\\hline
{}
\bottomrule
\end{{longtable}}

{}
"#,
                &cfd_case.to_pretty_string(),
                &cfd_case.to_string(),
                vort_pic,
                monitors
                    .force_latex_table(self.stats_time_range)
                    .zip(m1.force_latex_table(self.stats_time_range))
                    .map(|(x, y)| vec![x, y].join("\n"))
                    .unwrap_or_default(),
                m1_net
                    .force_latex_table(self.stats_time_range)
                    .unwrap_or_default(),
                path_to_case.join(format!("c-ring_parts{}", parts_suffix)),
                path_to_case.join(format!("m1-cell{}", parts_suffix)),
                path_to_case.join(format!("m1-segments{}", "")),
                path_to_case.join(format!("lower-truss{}", parts_suffix)),
                path_to_case.join(format!("upper-truss{}", parts_suffix)),
                path_to_case.join(format!("top-end{}", parts_suffix)),
                path_to_case.join(format!("m2-segments{}", parts_suffix)),
                path_to_case.join(format!("m12-baffles{}", parts_suffix)),
                path_to_case.join(format!("m1-outer-covers{}", parts_suffix)),
                path_to_case.join(format!("m1-inner-covers{}", parts_suffix)),
                path_to_case.join(format!("gir{}", parts_suffix)),
                path_to_case.join(format!("pfa-arms{}", parts_suffix)),
                path_to_case.join(format!("lgs{}", parts_suffix)),
                path_to_case.join(format!("platforms-cables{}", parts_suffix)),
                monitors
                    .moment_latex_table(self.stats_time_range)
                    .zip(m1.moment_latex_table(self.stats_time_range))
                    .map(|(x, y)| vec![x, y].join("\n"))
                    .unwrap_or_default(),
                m12_pressures
            ))
        } else {
            Ok(format!(
                r#"
\section{{{}}}
\label{{{}}}

\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}

\subsection{{Forces [N]}}
\begin{{longtable}}{{crrrr}}\toprule
 ELEMENT & MEAN & STD & MIN & MAX \\\hline
{}
\bottomrule
\end{{longtable}}

\subsubsection{{C-Rings}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M1 Cell}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{Lower trusses}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{Upper trusses}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{Top-end}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M2 segments}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M1 \& M2 baffles}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M1 outer covers}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{M1 inner covers}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{GIR}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{Prime focus assembly arms}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{Laser launch assemblies}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}
\subsubsection{{Platforms \& cable wraps}}
\includegraphics[width=0.8\textwidth]{{{{{{{:?}}}}}}}

\subsection{{Moments [N.M]}}
\begin{{longtable}}{{crrrr}}\toprule
 ELEMENT & MEAN & STD & MIN & MAX \\\hline
{}
\bottomrule
\end{{longtable}}

"#,
                &cfd_case.to_pretty_string(),
                &cfd_case.to_string(),
                vort_pic,
                monitors
                    .force_latex_table(self.stats_time_range)
                    .unwrap_or_default(),
                path_to_case.join("c-ring_parts"),
                path_to_case.join("m1-cell"),
                path_to_case.join("lower-truss"),
                path_to_case.join("upper-truss"),
                path_to_case.join("top-end"),
                path_to_case.join("m2-segments"),
                path_to_case.join("m12-baffles"),
                path_to_case.join("m1-outer-covers"),
                path_to_case.join("m1-inner-covers"),
                path_to_case.join("gir"),
                path_to_case.join("pfa-arms"),
                path_to_case.join("lgs"),
                path_to_case.join("platforms-cables"),
                monitors
                    .moment_latex_table(self.stats_time_range)
                    .unwrap_or_default(),
            ))
        }
    }
    /// Chapter assembly
    fn chapter(
        &self,
        zenith_angle: cfd::ZenithAngle,
        cfd_cases_subset: Option<&[cfd::CfdCase<2021>]>,
    ) -> Result<(), Box<dyn Error>> {
        let report_path = Path::new("report");
        let part = format!("part{}.", self.part);
        let chapter_filename = match zenith_angle {
            cfd::ZenithAngle::Zero => part + "chapter1.tex",
            cfd::ZenithAngle::Thirty => part + "chapter2.tex",
            cfd::ZenithAngle::Sixty => part + "chapter3.tex",
        };
        let cfd_cases = cfd::Baseline::<2021>::at_zenith(zenith_angle)
            .into_iter()
            .filter(|cfd_case| {
                if let Some(cases) = cfd_cases_subset {
                    cases.contains(cfd_case)
                } else {
                    true
                }
            })
            .collect::<Vec<cfd::CfdCase<2021>>>();
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
        String::from("Wind loads")
    }
}
impl WindLoads {
    /// Mount chapter assembly
    pub fn mount_chapter(&self, chapter_filename: Option<&str>) -> Result<(), Box<dyn Error>> {
        let report_path = Path::new("report");
        let cfd_cases = cfd::Baseline::<2021>::mount()
            .into_iter()
            .collect::<Vec<cfd::CfdCase<2021>>>();
        let results: Vec<_> = cfd_cases
            .into_par_iter()
            .map(|cfd_case| self.chapter_section(cfd_case, None).unwrap())
            .collect();
        let mut file =
            File::create(report_path.join(chapter_filename.unwrap_or("mount.chapter.tex")))?;
        write!(
            file,
            r#"
\chapter{{CFD Wind Loads}}
\label{{cfd-wind-loads}}
{}
"#,
            results.join("\n")
        )?;
        Ok(())
    }
}
