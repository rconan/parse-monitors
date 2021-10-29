use chrono::Local;
use parse_monitors::cfd::*;
use parse_monitors::{cfd, Band, DomeSeeing};
use rayon::prelude::*;
use std::time::Instant;
use std::{error::Error, fs::File, io::Write};
use strum::IntoEnumIterator;
use tectonic;

const CFD_YEAR: u32 = 2021;

fn dome_seeing(zenith_angle: ZenithAngle) -> String {
    let cfd_cases_21 = cfd::Baseline::<2021>::at_zenith(zenith_angle)
        .into_iter()
        .collect::<Vec<cfd::CfdCase<2021>>>();
    let results: Vec<_> = cfd_cases_21
        .into_par_iter()
        .map(|cfd_case_21| {
            let path_to_case = cfd::Baseline::<2021>::path().join(format!("{}", cfd_case_21));
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
    results
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
        .collect::<Vec<String>>()
        .join("\n")
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut zenith_chapters = vec![];
    println!("Assembling the report:");
    for zenith_angle in ZenithAngle::iter() {
        println!(" - {:?} zenith", &zenith_angle);
        zenith_chapters.push(format!(
            r#"
\chapter{{{}}}
\begin{{tabular}}{{*{{4}}{{c}}|*{{3}}{{r}}|*{{3}}{{r}}}}\toprule
 \multicolumn{{4}}{{c|}}{{\textbf{{CFD Cases}}}} & \multicolumn{{3}}{{|c|}}{{\textbf{{2021}}}} & \multicolumn{{3}}{{|c}}{{\textbf{{2020}}}} \\\midrule
  Zen. & Azi. & Cfg. & Wind & WFE & PSSn & PSSn & WFE & PSSn & PSSn \\
  - & -    & -    &  -   & RMS & -  & - & RMS & - & -  \\
  $[deg]$  & $[deg.]$ & - & $[m/s]$ & $[nm]$& V & H & $[nm]$ & V & H \\\hline
 {}
\bottomrule
\end{{tabular}}
"#,
            zenith_angle.clone().chapter_title(),
            dome_seeing(zenith_angle.clone())
        ));
        for cfd_case in Baseline::<CFD_YEAR>::at_zenith(zenith_angle).into_iter() {
            let path_to_case = Baseline::<CFD_YEAR>::path().join(&cfd_case.to_string());
            zenith_chapters.push(format!(
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
                //                    .join("TOTAL_FORCES.png")
                path_to_case.join("dome-seeing_wfe-rms.png"),
                path_to_case.join("dome-seeing_v-pssn.png"),
                path_to_case.join("dome-seeing_h-pssn.png"),
            ))
        }
    }

    let latex = format!(
        r#"
\documentclass{{report}}
\usepackage[colorlinks=true,linkcolor=blue]{{hyperref}}\usepackage{{graphicx}}
\usepackage{{booktabs}}

\addtolength{{\textwidth}}{{3cm}}
\addtolength{{\headheight}}{{5mm}}
\addtolength{{\evensidemargin}}{{-2cm}}
\addtolength{{\oddsidemargin}}{{-1cm}}

\title{{GMT CFD Baseline {}}}
\author{{R. Conan, K. Vogiatzis, H. Fitzpatrick}}
\date{{{:?}}}

\begin{{document}}
\maketitle
\tableofcontents
\listoffigures
\listoftables
{}
\end{{document}}
"#,
        CFD_YEAR,
        &Local::now().to_rfc2822(),
        zenith_chapters.join("\n"),
    );

    let now = Instant::now();
    println!("Compiling the report ...");
    let pdf_data: Vec<u8> = tectonic::latex_to_pdf(latex).expect("processing failed");
    let mut doc = File::create(format!("report/gmto.cfd{}.pdf", CFD_YEAR))?;
    doc.write_all(&pdf_data)?;
    println!(" ... report compiled in {}s", now.elapsed().as_secs());

    Ok(())
}
