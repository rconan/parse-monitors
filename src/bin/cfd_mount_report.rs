//use chrono::Local;
use parse_monitors::{report, report::Report};
use std::error::Error; //, fs::File, io::Write};
use std::time::Instant;
use structopt::StructOpt;
//use tectonic;

#[derive(Debug, StructOpt)]
#[structopt(name = "CFD 2021 Census", about = "Building 2021 CFD census report")]
struct Opt {
    //#[structopt(long)]
    //windloads: bool,
    /// Length of the time window used to compute the statistics counting from the end
    #[structopt(short = "d", long = "duration", default_value = "400")]
    stats_duration: f64,
}

const CFD_YEAR: u32 = 2021;

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    //    if opt.windloads {
    let now = Instant::now();
    println!("Building the report ...");
    report::WindLoads::new(0, opt.stats_duration)
        .exclude_monitors("floor|enclosure|screen|shutter")
        .keep_last(400)
        .detrend()
        .mount_chapter(Some("mount.detrend.chapter.tex"))?;
    println!(" ... report build in {}s", now.elapsed().as_secs());
    //    }

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

    \title{{GMT Observatory {} Computational Fluid Dynamics Mount Wind Loads}}
    \author{{R. Conan, K. Vogiatzis, H. Fitzpatrick}}
    \date{{{:?}}}

    \begin{{document}}
    \maketitle
    \tableofcontents

    \include{{report/mount.chapter}}

    \end{{document}}
    "#,
            CFD_YEAR,
            &Local::now().to_rfc2822(),
        );

        let now = Instant::now();
        println!("Compiling the report ...");
        let pdf_data: Vec<u8> = tectonic::latex_to_pdf(latex).expect("processing failed");
        let mut doc = File::create(format!("report/gmto.cfd{}.mount.pdf", CFD_YEAR))?;
        doc.write_all(&pdf_data)?;
        println!(" ... report compiled in {}s", now.elapsed().as_secs());
    */
    Ok(())
}
