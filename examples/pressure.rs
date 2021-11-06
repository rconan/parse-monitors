use glob::glob;
use indicatif::ParallelProgressIterator;
use parse_monitors::pressure::Pressure;
use parse_monitors::MonitorsLoader;
use parse_monitors::Vector;
use rayon::prelude::*;
use std::error::Error;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let paths = glob("/fsx/Baseline2021/Baseline2021/Baseline2021/CASES/zen30az000_OS7/pressures/M1p_M1p_*.csv.bz2")?;
    let files: Vec<_> = paths
        .map(|p| p.unwrap().to_str().unwrap().to_string())
        .collect();
    println!("Pressure files: {:?}", files.last().unwrap());
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
    let path = Path::new(files.last().unwrap());
    let csv_pressure = Pressure::decompress(path.to_path_buf())?;
    let csv_geometry = Pressure::decompress(path.with_file_name("M1p.csv.bz2"))?;
    let mut pressures = Pressure::load(csv_pressure, csv_geometry)?;
    let segments_force = pressures.segments_force();
    println!("Elapsed time: {}ms", now.elapsed().as_millis());
    println!("M1 Segments force: {:?}", segments_force);

    let monitors = MonitorsLoader::<2021>::default()
        .data_path("/fsx/Baseline2021/Baseline2021/Baseline2021/CASES/zen30az000_OS7")
        .header_filter("M1cell".to_string())
        .load()?;
    let keys: Vec<_> = monitors.forces_and_moments.keys().cloned().collect();
    let m1_cell_force = monitors.forces_and_moments["M1cell"]
        .last()
        .unwrap()
        .force
        .clone();
    println!("M1 cell force: {:}", m1_cell_force);
    let v: Vector = segments_force.into();
    println!("M1 total force: {:}", (&m1_cell_force + &v).unwrap());

    /*
    println!("x range: {:?}", pressures.x_range());
    println!("y range: {:?}", pressures.y_range());
    println!("z range: {:?}", pressures.z_range());

    println!("Segments force: {:?}", segments_force);
    println!("Total force: {:?}", segments_force.into_iter().sum::<f64>());
    println!("Total force: {:?}", pressures.total_force());
     */
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
