//! CREATE A REPORT ON A PARTICULAR SET OF CASES

use chrono::Local;
use parse_monitors::{cfd, report};
use std::{error::Error, fs::File, io::Write, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let special = String::from("thbound2");
    let now = Instant::now();
    println!("Building the report ...");
    if let Ok(chapter) = report::DomeSeeingPart::new(0, 0f64).special(
        &special,
        cfd::Baseline::<2021>::thbound2().into_iter().collect(),
    ) {
        println!(" ... report build in {}s", now.elapsed().as_secs());

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

    \setcounter{{tocdepth}}{{3}}

    \title{{CFD Thermal Boundary Updates}}
    \author{{R. Conan, K. Vogiatzis, H. Fitzpatrick}}
    \date{{{:?}}}

    \begin{{document}}
    \maketitle
%    \tableofcontents

    \include{{report/redo.chapter}}

    \end{{document}}
    "#,
            &Local::now().to_rfc2822(),
        );

        let now = Instant::now();
        println!("Compiling the report ...");
        let pdf_data: Vec<u8> = tectonic::latex_to_pdf(latex).expect("processing failed");
        let mut doc = File::create(format!("report/{}.pdf", special))?;
        doc.write_all(&pdf_data)?;
        println!(" ... report compiled in {}s", now.elapsed().as_secs());
    }
    Ok(())
}
