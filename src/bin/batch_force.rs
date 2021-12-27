//! Make all the wind forces plots
//!
//! It must be run as root i.e. `sudo -E ./target/release/batch_force`

use parse_monitors::{cfd::Baseline, Mirror, Monitors};
use rayon::prelude::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Truncate monitors to the `last` seconds
    #[structopt(short, long)]
    last: Option<usize>,
    /// Make all the plots
    #[structopt(long)]
    all: bool,
    /// Make C-Rings force magnitude plot
    #[structopt(long)]
    crings: bool,
    /// Make M1 cell force magnitude plot
    #[structopt(long)]
    m1_cell: bool,
    /// Make upper truss force magnitude plot
    #[structopt(long)]
    upper_truss: bool,
    /// Make lower truss force magnitude plot
    #[structopt(long)]
    lower_truss: bool,
    /// Make top-end force magnitude plot
    #[structopt(long)]
    top_end: bool,
    /// Make M1 segments force magnitude plot
    #[structopt(long)]
    m1_segments: bool,
    /// Make M2 segments force magnitude plot
    #[structopt(long)]
    m2_segments: bool,
    /// Make M1 and M2 baffles force magnitude plot
    #[structopt(long)]
    m12_baffles: bool,
    /// Make M1 inner mirror covers force magnitude plot
    #[structopt(long)]
    m1_inner_covers: bool,
    /// Make M1 outer mirror covers force magnitude plot
    #[structopt(long)]
    m1_outer_covers: bool,
    /// Make GIR force magnitude plot
    #[structopt(long)]
    gir: bool,
    /// Make PFA arms force magnitude plot
    #[structopt(long)]
    pfa_arms: bool,
    /// Make Laser Guide Stars assemblies force magnitude plot
    #[structopt(long)]
    lgsa: bool,
    /// Make platforms and cables force magnitude plot
    #[structopt(long)]
    platforms_cables: bool,
    /// Remove linear trends from monitors
    #[structopt(long)]
    detrend: bool,
}

const CFD_YEAR: u32 = 2021;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let cfd_root = Baseline::<CFD_YEAR>::path();
    let data_paths: Vec<_> = Baseline::<CFD_YEAR>::default()
        .into_iter()
        .map(|cfd_case| {
            cfd_root
                .join(cfd_case.to_string())
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();
    let n_cases = data_paths.len();
    println!("Found {} CFD cases", n_cases);

    let parts = vec![
        (opt.crings || opt.all).then(|| Some(("c-ring_parts", "Cring"))),
        (opt.m1_cell || opt.all).then(|| Some(("m1-cell", "M1cell"))),
        (opt.upper_truss || opt.all).then(|| Some(("upper-truss", "Tu"))),
        (opt.lower_truss || opt.all).then(|| Some(("lower-truss", "Tb"))),
        (opt.top_end || opt.all).then(|| Some(("top-end", "Top"))),
        (opt.m2_segments || opt.all).then(|| Some(("m2-segments", "M2s"))),
        (opt.m12_baffles || opt.all).then(|| Some(("m12-baffles", "Baf"))),
        (opt.m1_outer_covers || opt.all).then(|| Some(("m1-outer-covers", "M1cov[1-6]"))),
        (opt.m1_inner_covers || opt.all).then(|| Some(("m1-inner-covers", "M1covin[1-6]"))),
        (opt.gir || opt.all).then(|| Some(("gir", "GIR"))),
        (opt.pfa_arms || opt.all).then(|| Some(("pfa-arms", "arm"))),
        (opt.lgsa || opt.all).then(|| Some(("lgs", "LGS"))),
        (opt.platforms_cables || opt.all).then(|| Some(("platforms-cables", "cable|plat|level"))),
        opt.m1_segments.then(|| Some(("m1-segments", ""))),
    ];

    for part in parts.into_iter().filter_map(|x| x) {
        let (name, filter) = part.unwrap();
        println!("Part: {}", name);

        let _: Vec<_> = data_paths
            .par_iter()
            .map(|arg| {
                let mut filename = format!("{}/report/{}.png", arg, name);
                if name == "m1-segments" {
                    match Mirror::m1(arg).net_force().load() {
                        Ok(mut m1) => {
                            if let Some(arg) = opt.last {
                                m1.keep_last(arg);
                            }
                            m1.plot_forces(Some(filename.as_str()))
                        }
                        Err(e) => println!("{}: {:}", arg, e),
                    }
                } else {
                    let mut monitors = Monitors::loader::<String, CFD_YEAR>(arg.clone())
                        .header_filter(filter.to_string())
                        //.exclude_filter(xmon)
                        .load()
                        .unwrap();
                    if let Some(arg) = opt.last {
                        monitors.keep_last(arg);
                    }
                    if opt.detrend {
                        monitors.detrend();
                        filename = format!("{}/{}-detrend.png", arg, name)
                    }
                    monitors.plot_forces(Some(filename.as_str()));
                }
            })
            .collect();
    }

    Ok(())
}
