//! Make all the wind forces plots
//!
//! It must be run as root i.e. `sudo -E ./../target/release/batch_force --all`

use std::{fs::create_dir, path::Path};

use clap::Parser;
use parse_monitors::{cfd::Baseline, cfd::BaselineTrait, Mirror, Monitors};
use rayon::prelude::*;

#[derive(Debug, Parser)]
struct Cli {
    /// Truncate monitors to the `last` seconds
    #[arg(short, long)]
    last: Option<usize>,
    /// Make all the plots
    #[arg(long)]
    all: bool,
    /// Make C-Rings force magnitude plot
    #[arg(long)]
    crings: bool,
    /// Make M1 cell force magnitude plot
    #[arg(long)]
    m1_cell: bool,
    /// Make upper truss force magnitude plot
    #[arg(long)]
    upper_truss: bool,
    /// Make lower truss force magnitude plot
    #[arg(long)]
    lower_truss: bool,
    /// Make top-end force magnitude plot
    #[arg(long)]
    top_end: bool,
    /// Make M1 segments force magnitude plot
    #[arg(long)]
    m1_segments: bool,
    /// Make M2 segments force magnitude plot
    #[arg(long)]
    m2_segments: bool,
    /// Make M1 and M2 baffles force magnitude plot
    #[arg(long)]
    m12_baffles: bool,
    /// Make M1 inner mirror covers force magnitude plot
    #[arg(long)]
    m1_inner_covers: bool,
    /// Make M1 outer mirror covers force magnitude plot
    #[arg(long)]
    m1_outer_covers: bool,
    /// Make GIR force magnitude plot
    #[arg(long)]
    gir: bool,
    /// Make PFA arms force magnitude plot
    #[arg(long)]
    pfa_arms: bool,
    /// Make Laser Guide Stars assemblies force magnitude plot
    #[arg(long)]
    lgsa: bool,
    /// Make platforms and cables force magnitude plot
    #[arg(long)]
    platforms_cables: bool,
    /// Remove linear trends from monitors
    #[arg(long)]
    detrend: bool,
}

const CFD_YEAR: u32 = 2025;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let opt = Cli::parse();
    let cfd_root = Baseline::<CFD_YEAR>::path();
    let data_paths: Vec<_> = Baseline::<CFD_YEAR>::from_env()
        .unwrap_or_default()
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
        // (opt.m1_cell || opt.all).then(|| Some(("m1-cell", "M1cell"))),
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
                let path = Path::new(arg).join("report");
                if !path.is_dir() {
                    create_dir(&path).expect(&format!("Failed to create dir: {:?}", path))
                }
                let mut filename = path.join(name).with_extension("png");
                log::info!("{filename:?}");
                if name == "m1-segments" {
                    match Mirror::m1(arg).net_force().load() {
                        Ok(mut m1) => {
                            if let Some(arg) = opt.last {
                                m1.keep_last(arg);
                            }
                            m1.plot_forces(filename.to_str())
                        }
                        Err(e) => println!("{}: {:}", arg, e),
                    }
                } else {
                    let mut monitors = Monitors::loader::<String, CFD_YEAR>(arg.clone())
                        .header_filter(filter)
                        //.exclude_filter(xmon)
                        .load()
                        .unwrap();
                    if let Some(arg) = opt.last {
                        monitors.keep_last(arg);
                    }
                    if opt.detrend {
                        monitors.detrend();
                        filename = path
                            .join(format!("{}-detrend.png", name))
                            .with_extension("png");
                    }
                    monitors.plot_forces(filename.to_str()).unwrap();
                }
            })
            .collect();
    }

    Ok(())
}
