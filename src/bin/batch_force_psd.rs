//! Make all the wind forces plots
//!
//! It must be run as root i.e. `sudo -E ./../target/release/batch_force --all`

use std::{env, fs::create_dir, path::Path};

use clap::Parser;
use parse_monitors::{
    cfd::{self, Baseline, BaselineTrait, CfdCase},
    Mirror, Monitors,
};
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
    /// skip that many CFD cases
    #[arg(short, long)]
    skip: Option<usize>,
    /// process only that many CFD cases
    #[arg(short, long)]
    take: Option<usize>,
    /// folder where the plot with be saved
    #[arg(long)]
    to: Option<String>,
}

const CFD_YEAR: u32 = 2025;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    match (cli.skip, cli.take) {
        (None, None) => Box::new(cfd::Baseline::<CFD_YEAR>::default().into_iter())
            as Box<dyn Iterator<Item = CfdCase<{ CFD_YEAR }>>>,
        (None, Some(t)) => Box::new(cfd::Baseline::<CFD_YEAR>::default().into_iter().take(t))
            as Box<dyn Iterator<Item = CfdCase<{ CFD_YEAR }>>>,
        (Some(s), None) => Box::new(cfd::Baseline::<CFD_YEAR>::default().into_iter().skip(s))
            as Box<dyn Iterator<Item = CfdCase<{ CFD_YEAR }>>>,
        (Some(s), Some(t)) => Box::new(
            cfd::Baseline::<CFD_YEAR>::default()
                .into_iter()
                .skip(s)
                .take(t),
        ) as Box<dyn Iterator<Item = CfdCase<{ CFD_YEAR }>>>,
    }
    .for_each(|cfd_case| task(cfd_case, &cli).expect(&format!("{cfd_case:?} failed")));

    Ok(())
}
fn task(cfd_case: CfdCase<{ CFD_YEAR }>, opt: &Cli) -> anyhow::Result<()> {
    println!("{cfd_case}");
    let cfd_root = Baseline::<CFD_YEAR>::path()?;

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

        let to = opt.to.as_ref().map_or(".".to_string(), |x| x.to_owned());
        let path = Path::new(&to);
        let mut monitors = Monitors::loader::<String, CFD_YEAR>(
            cfd_root
                .join(&cfd_case.to_string())
                .to_str()
                .unwrap()
                .to_string(),
        )
        .header_filter(filter)
        //.exclude_filter(xmon)
        .load()
        .unwrap();
        if let Some(arg) = opt.last {
            monitors.keep_last(arg);
        }
        for (component, exertion) in &monitors.forces_and_moments {
            let forces: Vec<_> = exertion
                .into_iter()
                .map(|exertion| exertion.force.clone())
                .collect();
            let mut filepath = path
                .join(format!("{}_psd", component))
                .with_extension("png");
            let plot = complot::Config::new()
                .filename(filepath.to_str().unwrap())
                .xaxis(complot::Axis::new().label("Frequency [Hz]"))
                .yaxis(complot::Axis::new().label("FORCE PSD [N^2/Hz]"))
                .legend(vec!["Fx", "Fy", "Fz"]);
            Monitors::plot_this_forces_psds(&forces, Some(plot));
        }
        // if opt.detrend {
        //     monitors.detrend();
        //     filename = path
        //         .join(format!("{}-detrend.png", name))
        //         .with_extension("png");
        // }
        // monitors.plot_forces(filename.to_str()).unwrap();
        // }
    }

    Ok(())
}
