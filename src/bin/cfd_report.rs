use chrono::Local;
use parse_monitors::cfd::*;
use std::{error::Error, fs::File, io::Write, iter::once};
use strum::IntoEnumIterator;
use tectonic;

const CFD_YEAR: u32 = 2020;

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
            zenith_chapters.push(format!(
                r#"
\clearpage
\section{{{}}}

\includegraphics[width=\textwidth]{{{:?}}}
"#,
                &cfd_case.to_pretty_string(),
                Baseline::<CFD_YEAR>::path()
                    .join(&cfd_case.to_string())
                    .join("TOTAL_FORCES.png")
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

    let pdf_data: Vec<u8> = tectonic::latex_to_pdf(latex).expect("processing failed");
    let mut doc = File::create(format!("report/gmto.cfd{}.pdf", CFD_YEAR))?;
    doc.write_all(&pdf_data)?;

    Ok(())
}
