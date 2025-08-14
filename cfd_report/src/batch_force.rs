//! Make all the wind forces plots
//!
//! It must be run as root i.e. `sudo -E ./../target/release/batch_force --all`

use std::{fs::create_dir, path::Path};

use parse_monitors::{
    Mirror, Monitors,
    cfd::{Baseline, BaselineTrait, CfdCase},
};
use rayon::prelude::*;

use crate::{ForcesCli, ReportError};

pub fn task<const Y: u32>(cfd_cases: &[CfdCase<Y>], opt: ForcesCli) -> Result<(),ReportError> {
    let cfd_root = Baseline::<Y>::path()?;
    let data_paths: Vec<_> = cfd_cases
        .into_iter()
        .map(|cfd_case| {
            cfd_root
                .join(cfd_case.to_string())
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();
    // let n_cases = data_paths.len();
    // println!("Found {} CFD cases", n_cases);

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
        // println!("Part: {}", name);

        let _: Vec<_> = data_paths
            .par_iter()
            .skip(38)
            .take(2)
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
                    let mut monitors = Monitors::loader::<String, Y>(arg.clone())
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
