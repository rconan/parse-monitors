use glob::glob;
use indicatif::ParallelProgressIterator;
use parse_monitors::pressure::Pressure;
use rayon::prelude::*;
use std::error::Error;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let paths = glob("/fsx/Baseline2021/Baseline2021/Baseline2021/CASES/zen30az000_OS7/pressures/M1p_M1p_*.csv.bz2")?;
    let files: Vec<_> = paths
        .take(1)
        .map(|p| p.unwrap().to_str().unwrap().to_string())
        .collect();
    println!("Pressure files: {:?}", files);
    /*
        let now = Instant::now();
        let total_absolute_force: Vec<_> = files
            .iter()
            // .progress_count(files.len() as u64)
            .map(|f| {
                let pressures = Pressure::load(f).unwrap();
                pressures.total_absolute_force()
            })
            .collect();
        println!("Elapsed time: {}ms", now.elapsed().as_millis());
        println!("Total force: {:?}", total_absolute_force);
    */
    let now = Instant::now();
    let mut pressures = Pressure::load(&files[0]).unwrap();
    let segments_force = pressures.segments_force();
    println!("Elapsed time: {}ms", now.elapsed().as_millis());

    println!("x range: {:?}", pressures.x_range());
    println!("y range: {:?}", pressures.y_range());
    println!("z range: {:?}", pressures.z_range());

    println!("Segments force: {:?}", segments_force);
    println!("Total force: {:?}", segments_force.into_iter().sum::<f64>());
    println!("Total force: {:?}", pressures.total_force());
    /*
        &pressures
            .to_local(7)
            .xy_iter()
            .filter_map(|(x, y)| {
                if x.hypot(*y) < 4.5_f64 {
                    Some((x, y))
                } else {
                    None
                }
            })
            .map(|(x, y)| (*x, vec![*y]))
            .collect::<complot::Scatter>();
    */
    Ok(())
}
