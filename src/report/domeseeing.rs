use crate::{
    cfd::{self, Baseline, CfdCase},
    Band, DomeSeeing,
};
use rayon::prelude::*;
use std::{error::Error, fs::File, io::Write, path::Path};
use strum::IntoEnumIterator;
/// Chapter table
pub fn chapter_table(zenith_angle: cfd::ZenithAngle) -> String {
    let cfd_cases_21 = cfd::Baseline::<{ super::CFD_YEAR }>::at_zenith(zenith_angle.clone())
        .into_iter()
        .collect::<Vec<cfd::CfdCase<{ super::CFD_YEAR }>>>();
    let results: Vec<_> = cfd_cases_21
        .into_par_iter()
        .map(|cfd_case_21| {
            let path_to_case =
                cfd::Baseline::<{ super::CFD_YEAR }>::path().join(format!("{}", cfd_case_21));
            let ds_21 = DomeSeeing::load(path_to_case.clone()).unwrap();
            if let (Some(v_pssn), Some(h_pssn)) = (ds_21.pssn(Band::V), ds_21.pssn(Band::H)) {
                let wfe_rms =
                    1e9 * (ds_21.wfe_rms().map(|x| x * x).sum::<f64>() / ds_21.len() as f64).sqrt();
                Some((
                    (cfd_case_21.clone(), wfe_rms, v_pssn, h_pssn),
                    if let Some(cfd_case_20) = cfd::Baseline::<2020>::find(cfd_case_21) {
                        let ds_20 = DomeSeeing::load(
                            cfd::Baseline::<2020>::path().join(format!("{}", cfd_case_20)),
                        )
                        .unwrap();
                        if let (Some(v_pssn), Some(h_pssn)) =
                            (ds_20.pssn(Band::V), ds_20.pssn(Band::H))
                        {
                            let wfe_rms = 1e9
                                * (ds_20.wfe_rms().map(|x| x * x).sum::<f64>()
                                    / ds_20.len() as f64)
                                    .sqrt();
                            Some((cfd_case_20, wfe_rms, v_pssn, h_pssn))
                        } else {
                            None
                        }
                    } else {
                        None
                    },
                ))
            } else {
                None
            }
        })
        .collect();
    let table_content = results
        .into_iter()
        .map(|result| match result {
            Some((
                (cfd_case_21, wfe_rms_21, v_pssn_21, h_pssn_21),
                Some((_, wfe_rms_20, v_pssn_20, h_pssn_20)),
            )) => {
                format!(
                    r#" {:} & {:6.0} & {:.4} & {:.4} & {:6.0} & {:.4} & {:.4} \\"#,
                    cfd_case_21.to_latex_string(),
                    wfe_rms_21,
                    v_pssn_21,
                    h_pssn_21,
                    wfe_rms_20,
                    v_pssn_20,
                    h_pssn_20
                )
            }
            Some(((cfd_case_21, wfe_rms_21, v_pssn_21, h_pssn_21), None)) => {
                format!(
                    r#" {:} & {:6.0} & {:.4} & {:.4} \\"#,
                    cfd_case_21.to_latex_string(),
                    wfe_rms_21,
                    v_pssn_21,
                    h_pssn_21,
                )
            }
            _ => unimplemented!(),
        })
        .collect::<Vec<String>>();
    format!(
        r#"
\begin{{tabular}}{{*{{4}}{{c}}|*{{3}}{{r}}|*{{3}}{{r}}}}\toprule
 \multicolumn{{4}}{{c|}}{{\textbf{{CFD Cases}}}} & \multicolumn{{3}}{{|c|}}{{\textbf{{2021}}}} & \multicolumn{{3}}{{|c}}{{\textbf{{2020}}}} \\\midrule
  Zen. & Azi. & Cfg. & Wind & WFE & PSSn & PSSn & WFE & PSSn & PSSn \\
  - & -    & -    &  -   & RMS & -  & - & RMS & - & -  \\
  $[deg]$  & $[deg.]$ & - & $[m/s]$ & $[nm]$& V & H & $[nm]$ & V & H \\\hline
 {}
\bottomrule
\end{{tabular}}
"#,
        table_content.join("\n")
    )
}
/// Chapter section
pub fn chapter_section(cfd_case: CfdCase<{ super::CFD_YEAR }>) -> String {
    let path_to_case = Baseline::<{ super::CFD_YEAR }>::path().join(&cfd_case.to_string());
    format!(
        r#"
\clearpage
\section{{{}}}

\subsection{{Wavefront error RMS}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\clearpage
\subsection{{PSSn}}
\subsubsection{{V}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
\subsubsection{{H}}
\includegraphics[width=0.8\textwidth]{{{:?}}}
"#,
        &cfd_case.to_pretty_string(),
        path_to_case.join("dome-seeing_wfe-rms.png"),
        path_to_case.join("dome-seeing_v-pssn.png"),
        path_to_case.join("dome-seeing_h-pssn.png"),
    )
}
/// Chapter assembly
pub fn chapter(zenith_angle: cfd::ZenithAngle) -> Result<(), Box<dyn Error>> {
    let report_path = Path::new("report");
    let chapter_filename = match zenith_angle {
        cfd::ZenithAngle::Zero => "part1.chapter1.tex",
        cfd::ZenithAngle::Thirty => "part1.chapter2.tex",
        cfd::ZenithAngle::Sixty => "part1.chapter3.tex",
    };
    let mut file = File::create(report_path.join(chapter_filename))?;
    write!(
        file,
        r#"
\chapter{{{}}}
{},
{}
"#,
        zenith_angle.chapter_title(),
        chapter_table(zenith_angle.clone()),
        cfd::Baseline::<{ super::CFD_YEAR }>::at_zenith(zenith_angle)
            .into_iter()
            .map(chapter_section)
            .collect::<Vec<String>>()
            .join("\n")
    )?;
    Ok(())
}
/// Part assembly
pub fn part() -> Result<(), Box<dyn Error>> {
    println!(" -->> dome seeing ...");
    for zenith_angle in cfd::ZenithAngle::iter() {
        chapter(zenith_angle)?;
    }
    Ok(())
}
