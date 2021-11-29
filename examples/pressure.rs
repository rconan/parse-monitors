use glob::glob;
use indicatif::ParallelProgressIterator;
use parse_monitors::{pressure::Pressure, Mirror, MonitorsLoader, Vector};
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

    let segments_pressure = pressures.segments_average_pressure();
    let segments_pressure_alt: Vec<_> = (1..=7)
        .map(|sid| -> Vector { pressures.segment_force(sid).into() })
        .collect::<Vec<Vector>>()
        .into_iter()
        .zip(pressures.segments_area())
        .map(|(f, a)| f.magnitude().unwrap() / a)
        .collect();
    let segments_pressure_std = pressures.segments_pressure_std();
    let average_pressure = pressures.mirror_average_pressure();
    let average_pressure_alt = segments_pressure_alt
        .iter()
        .zip(pressures.segments_area())
        .fold(0f64, |pa, (p, a)| pa + *p * a)
        / pressures.mirror_area();
    pressures.pressure_map();
    println!("Elapsed time: {}ms", now.elapsed().as_millis());
    println!("Segment pressure: {:.3?}Pa", segments_pressure);
    println!("Segment pressure: {:.3?}Pa", segments_pressure_alt);
    println!("Segment pressure std.: {:.3?}Pa", segments_pressure_std);
    println!("Mirror pressure : {:.3?}Pa", average_pressure);
    println!("Mirror pressure : {:.3?}Pa", average_pressure_alt);

    /*
    let now = Instant::now();
    let path = Path::new(files.last().unwrap());
    let csv_pressure = Pressure::decompress(path.to_path_buf())?;
    let csv_geometry = Pressure::decompress(path.with_file_name("M1p.csv.bz2"))?;
    let mut pressures = Pressure::load(csv_pressure, csv_geometry)?;
    let (cop, fm): (Vec<_>, Vec<_>) = (1..=7)
        .map(|sid| pressures.segment_pressure_integral(sid))
        .unzip();
    println!("Elapsed time: {}ms", now.elapsed().as_millis());

    let segments_force = pressures.segments_force();
    println!("Elapsed time: {}ms", now.elapsed().as_millis());
    println!("M1 Segments force: {:?}", segments_force);
    let (cop, fm): (Vec<_>, Vec<_>) = (1..=7)
        .map(|sid| pressures.segment_pressure_integral(sid))
        .unzip();
    let (f, m): (Vec<_>, Vec<_>) = fm.into_iter().unzip();
    println!("Sum forces : {:6.3?}", pressures.segments_force());
    println!("Sum forces : {:6.3?}", pressures.sum_vectors(f.iter()));
    println!("Sum moments: {:6.3?}", pressures.sum_vectors(m.iter()));

    println!("q {}", cop[4][1] * f[4][2] - cop[4][2] * f[4][1]);

    let monitors = MonitorsLoader::<2021>::default()
        .data_path("/fsx/Baseline2021/Baseline2021/Baseline2021/CASES/zen30az000_OS7")
        .header_filter("M1cell".to_string())
        .load()?;
    let pos = monitors
        .time
        .iter()
        .position(|&t| (t - 700f64).abs() < 20f64.recip())
        .unwrap();
    println!("Time (last pressure): {:}", monitors.time[pos]);
    let keys: Vec<_> = monitors.forces_and_moments.keys().cloned().collect();
    let m1_cell_force = monitors.forces_and_moments["M1cell"][pos].force.clone();
    println!("M1 cell force: {:}", m1_cell_force);
    let m1_cell_moment = monitors.forces_and_moments["M1cell"][pos].moment.clone();
    println!("M1 cell moment: {:}", m1_cell_moment);
    let v: Vector = segments_force.into();
    println!("M1 total force: {:}", (&m1_cell_force + &v).unwrap());

    println!("Segments force: {:?}", segments_force);
    println!("Total force: {:?}", segments_force.into_iter().sum::<f64>());
    println!("Total force: {:?}", pressures.total_force());
     */

    println!("x range: {:?}", pressures.x_range());
    println!("y range: {:?}", pressures.y_range());
    println!("z range: {:?}", pressures.z_range());

    &pressures
        .to_local(1)
        .xy_iter()
        .filter_map(|(x, y)| {
            if x.hypot(y) < 0.55_f64 {
                Some((x, y))
            } else {
                None
            }
        })
        .map(|(x, y)| (x, vec![y]))
        .collect::<complot::Scatter>();

    /*
        {
            let mut m1 = Mirror::m1("/fsx/Baseline2021/Baseline2021/Baseline2021/CASES/zen30az000_OS7")
                .net_force()
                .load()
                .unwrap();
            let pos = m1
                .time()
                .iter()
                .position(|&t| (t - 700f64).abs() < 40f64.recip())
                .unwrap();
            let t = m1.time()[pos];
            let (total_force, total_moment) =
                m1.exertion()
                    .fold((Vector::zero(), Vector::zero()), |(mut f, mut m), e| {
                        let mut q = &mut f;
                        q += &e[pos].force;
                        let mut q = &mut m;
                        q += &e[pos].moment;
                        (f, m)
                    });
            println!("{}: {:}", t, total_force);
            println!("{}: {:}", t, total_moment);
        }
    */
    Ok(())
}
