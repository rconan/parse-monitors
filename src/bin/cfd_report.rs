use chrono::Local;
use parse_monitors::cfd::*;
use std::time::Instant;
use std::{error::Error, fs::File, io::Write};
use strum::IntoEnumIterator;
use tectonic;

const CFD_YEAR: u32 = 2021;

fn main() -> Result<(), Box<dyn Error>> {
    let mut zenith_chapters = vec![];
    for zenith_angle in ZenithAngle::iter() {
        zenith_chapters.push(format!(
            r#"
\chapter{{{}}}
"#,
            zenith_angle.clone().chapter_title(),
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
