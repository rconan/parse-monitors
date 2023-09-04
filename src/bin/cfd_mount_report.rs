//use chrono::Local;
use parse_monitors::{cfd, report, report::Report};
use std::error::Error; //, fs::File, io::Write};
use std::time::Instant;
use structopt::StructOpt;

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
    let now = Instant::now();
    println!("Building the report ...");
    report::WindLoads::new(0, opt.stats_duration)
        .exclude_monitors("floor|enclosure|screen|shutter|M1level")
        .keep_last(400)
        .cfd_case(cfd::CfdCase::colloquial(30, 135, "nos", 7)?)
        //.detrend()
        .mount_chapter(Some("mount.chapter.tex"))?;
    println!(" ... report build in {}s", now.elapsed().as_secs());
    Ok(())
}
