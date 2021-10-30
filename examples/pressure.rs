use glob::glob;
use indicatif::ParallelProgressIterator;
use parse_monitors::pressure::Pressure;
use rayon::prelude::*;
use std::error::Error;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let paths = glob("/fsx/Baseline2021/Baseline2021/Baseline2021/CASES/zen30az000_OS7/pressures/M1p_M1p_*.csv.bz2")?;
    let files: Vec<_> = paths
        .map(|p| p.unwrap().to_str().unwrap().to_string())
        .collect();
    let now = Instant::now();
    let total_absolute_force: Vec<_> = files
        .par_iter()
        .progress_count(files.len() as u64)
        .map(|f| {
            let pressures = Pressure::load(f).unwrap();
            pressures.total_absolute_force()
        })
        .collect();
    println!("Elapsed time: {}ms", now.elapsed().as_millis());

    /*   pressures
            .x_iter()
            .zip(pressures.y_iter())
            .map(|(x, y)| (*x, vec![*y]))
            .collect::<complot::Scatter>();
    */
    Ok(())
}
