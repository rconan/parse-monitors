//! CFD REPORT

//use chrono::Local;
use parse_monitors::{report, report::Report};
use std::error::Error; //, fs::File, io::Write};
use std::time::Instant;
use structopt::StructOpt;
//use tectonic;
use std::thread;

#[derive(Debug, StructOpt)]
#[structopt(name = "CFD 2021 Census", about = "Building 2021 CFD census report")]
struct Opt {
    #[structopt(long)]
    full: bool,
    #[structopt(long)]
    domeseeing: bool,
    #[structopt(long)]
    windloads: bool,
    #[structopt(long)]
    htc: bool,
}

//const CFD_YEAR: u32 = 2021;

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    let mut tjh = vec![];
    let now = Instant::now();
    println!("Building the different parts of the report ...");
    if opt.domeseeing || opt.full {
        tjh.push(thread::spawn(|| {
            report::DomeSeeingPart::new(1, 0f64).part().unwrap();
        }));
    }
    if opt.htc || opt.full {
        tjh.push(thread::spawn(|| {
            report::HTC::new(3, 400f64).part().unwrap();
        }));
    }
    if opt.windloads || opt.full {
        tjh.push(thread::spawn(|| {
            report::WindLoads::new(2, 400f64).part().unwrap();
        }));
    }

    for t in tjh {
        t.join().unwrap();
    }

    println!(" ... report parts build in {}s", now.elapsed().as_secs());

    /*
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

    \title{{GMT Observatory {} Computational Fluid Dynamics Census}}
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
    */
    Ok(())
}
