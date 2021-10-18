use glob::glob;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use parse_monitors::{plot_monitor, MonitorsLoader};
use rayon::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let xmon = "floor|shutter|screen|enclosure";
    let data_paths: Vec<String> = glob("data/zen*")?
        .map(|p| p.unwrap().to_str().unwrap().to_string())
        .collect();
    let pb = ProgressBar::new(data_paths.len() as u64);
    let _: Vec<_> = data_paths
        .par_iter()
        .progress_with(pb)
        .map(|arg| {
            let mut monitors = MonitorsLoader::default()
                .data_path(arg.clone())
                .exclude_filter(xmon)
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
