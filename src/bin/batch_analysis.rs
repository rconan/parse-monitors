use indicatif::{ParallelProgressIterator, ProgressBar};
use parse_monitors::{cfd::Baseline, plot_monitor, MonitorsLoader};
use rayon::prelude::*;
use std::path::Path;

const CFD_YEAR: u32 = 2020;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    //    let xmon = "floor|shutter|screen|enclosure";
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
    let pb = ProgressBar::new(n_cases as u64);
    let _: Vec<_> = data_paths
        .par_iter()
        .progress_with(pb)
        .map(|arg| {
            let mut monitors = MonitorsLoader::<CFD_YEAR>::default()
                .data_path(arg.clone())
                .header_filter("M1cov|T|M2".to_string())
                //.exclude_filter(xmon)
                .load()
                .unwrap();
            monitors.total_exertion();
            plot_monitor(
                &monitors.time,
                &monitors.total_forces_and_moments,
                "Total",
                arg,
            );
        })
        .collect();
    Ok(())
}
