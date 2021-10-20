use glob::glob;
use indicatif::{ParallelProgressIterator, ProgressBar};
use parse_monitors::{plot_monitor, MonitorsLoader};
use rayon::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    //    let xmon = "floor|shutter|screen|enclosure";
    let data_paths: Vec<String> = glob("/fsx/Baseline2021/Baseline2021/Baseline2021/CASES/zen*")?
        .map(|p| p.unwrap().to_str().unwrap().to_string())
        .collect();
    let n_cases = data_paths.len();
    println!("Found {} CFD cases", n_cases);
    let pb = ProgressBar::new(n_cases as u64);
    let _: Vec<_> = data_paths
        .par_iter()
        .progress_with(pb)
        .map(|arg| {
            let mut monitors = MonitorsLoader::default()
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
