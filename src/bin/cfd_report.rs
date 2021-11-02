use chrono::Local;
use parse_monitors::{report, report::Report};
use std::time::Instant;
use std::{error::Error, fs::File, io::Write};
use tectonic;

const CFD_YEAR: u32 = 2021;

fn main() -> Result<(), Box<dyn Error>> {
    let now = Instant::now();
    println!("Building the different parts of the report ...");
    //    report::dome_seeing::part()?;
    println!(" -->> HTC ...");
    report::HTC::new(3, 400f64).part()?;
    println!(" -->> wind loads ...");
    report::WindLoads::new(2, 400f64).part()?;
    println!(" ... report parts build in {}s", now.elapsed().as_secs());

    let latex = format!(
        r#"
\documentclass{{report}}
\usepackage[colorlinks=true,linkcolor=blue]{{hyperref}}\usepackage{{graphicx}}
\usepackage{{booktabs}}
\usepackage{{longtable}}

\addtolength{{\textwidth}}{{3cm}}
\addtolength{{\headheight}}{{5mm}}
\addtolength{{\evensidemargin}}{{-2cm}}
\addtolength{{\oddsidemargin}}{{-1cm}}

\title{{GMT {} Computational Fluid Dynamics Census}}
\author{{R. Conan, K. Vogiatzis, H. Fitzpatrick}}
\date{{{:?}}}

\begin{{document}}
\maketitle
\tableofcontents
\listoffigures
\listoftables

\part{{Dome Seeing}}

\include{{report/part1.chapter1}}
\include{{report/part1.chapter2}}
\include{{report/part1.chapter3}}

\part{{Wind Loads}}

\include{{report/part2.chapter1}}
\include{{report/part2.chapter2}}
\include{{report/part2.chapter3}}

\part{{Heat Transfer Coefficients}}

\include{{report/part3.chapter1}}
\include{{report/part3.chapter2}}
\include{{report/part3.chapter3}}

\end{{document}}
"#,
        CFD_YEAR,
        &Local::now().to_rfc2822(),
    );

    let now = Instant::now();
    println!("Compiling the report ...");
    let pdf_data: Vec<u8> = tectonic::latex_to_pdf(latex).expect("processing failed");
    let mut doc = File::create(format!("report/gmto.cfd{}.pdf", CFD_YEAR))?;
    doc.write_all(&pdf_data)?;
    println!(" ... report compiled in {}s", now.elapsed().as_secs());

    Ok(())
}
