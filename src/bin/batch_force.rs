use indicatif::{ParallelProgressIterator, ProgressBar};
use parse_monitors::{cfd::Baseline, plot_monitor, MonitorsLoader};
use rayon::prelude::*;
use std::path::Path;

const CFD_YEAR: u32 = 2021;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    //let xmon = "floor|shutter|screen|enclosure";
    let cfd_root = match CFD_YEAR {
        2020_u32 => Path::new("/fsx/Baseline2020"),
        2021_u32 => Path::new("/fsx/Baseline2021/Baseline2021/Baseline2021/CASES"),
        _ => panic!("Not a good year!"),
    };
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

    let (name, filter) = ("c-ring_parts", "Cring");
    //let (name, filter) = ("m1-cell", "M1cell");
    //let (name, filter) = ("upper-truss", "Tu");
    //let (name, filter) = ("lower-truss", "Tb");
    //let (name, filter) = ("top-end", "Top");
    //let (name, filter) = ("m2-segments", "M2s");
    //let (name, filter) = ("m12-baffles", "Baf");
    //let (name, filter) = ("m1-outer-covers", "M1cov[1-6]");
    //let (name, filter) = ("m1-inner-covers", "M1covin[1-6]");
    //let (name, filter) = ("gir", "GIR");
    //let (name, filter) = ("pfa-arms", "arm");
    //let (name, filter) = ("lgs", "LGS");
    //let (name, filter) = ("platforms-cables", "cable|plat|level");

    let pb = ProgressBar::new(n_cases as u64);
    let _: Vec<_> = data_paths
        .par_iter()
        .progress_with(pb)
        .map(|arg| {
            let monitors = MonitorsLoader::<CFD_YEAR>::default()
                .data_path(arg.clone())
                .header_filter(filter.to_string())
                //.exclude_filter(xmon)
                .load()
                .unwrap();
            let filename = format!("{}/{}.png", arg, name);
            monitors.plot_forces(Some(filename.as_str()));
        })
        .collect();
    Ok(())
}
