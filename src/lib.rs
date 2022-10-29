//! GMT Computation Fluid Dynamic Post Processing API
//!
//! A library to query and to process the GMT CFD Baseline databases.

use nalgebra as na;

pub mod error;
mod vector;
pub use vector::Vector;
mod monitors;
pub use monitors::{Exertion, Mirror, Monitors, MonitorsError, MonitorsLoader};
pub mod cfd;
pub mod domeseeing;
pub use domeseeing::{Band, DomeSeeing};
pub mod pressure;
pub mod report;
pub mod temperature;

pub const FORCE_SAMPLING_FREQUENCY: f64 = 20_f64; // Hz
pub const FORCE_SAMPLING: f64 = 1. / FORCE_SAMPLING_FREQUENCY; // Hz
pub const TEMPERATURE_SAMPLING_FREQUENCY: f64 = 5_f64; // Hz

pub fn polyfit<T: na::RealField + Copy>(
    x_values: &[T],
    y_values: &[T],
    polynomial_degree: usize,
) -> Result<Vec<T>, &'static str> {
    let number_of_columns = polynomial_degree + 1;
    let number_of_rows = x_values.len();
    let mut a = na::DMatrix::zeros(number_of_rows, number_of_columns);

    for (row, &x) in x_values.iter().enumerate() {
        // First column is always 1
        a[(row, 0)] = T::one();

        for col in 1..number_of_columns {
            a[(row, col)] = x.powf(na::convert(col as f64));
        }
    }

    let b = na::DVector::from_row_slice(y_values);

    let decomp = na::SVD::new(a, true, true);

    match decomp.solve(&b, na::convert(1e-18f64)) {
        Ok(mat) => Ok(mat.data.into()),
        Err(error) => Err(error),
    }
}
pub fn detrend<T: na::RealField + Copy>(
    x_values: &[T],
    y_values: &[T],
    polynomial_degree: usize,
) -> Result<Vec<T>, &'static str> {
    let number_of_columns = polynomial_degree + 1;
    let number_of_rows = x_values.len();
    let mut a = na::DMatrix::zeros(number_of_rows, number_of_columns);

    for (row, &x) in x_values.iter().enumerate() {
        // First column is always 1
        a[(row, 0)] = T::one();

        for col in 1..number_of_columns {
            a[(row, col)] = x.powf(na::convert(col as f64));
        }
    }

    let b = na::DVector::from_row_slice(y_values);

    let decomp = na::SVD::new(a.clone(), true, true);

    match decomp.solve(&b, na::convert(1e-18f64)) {
        Ok(mat) => {
            let y_detrend = b - a * &mat;
            Ok(y_detrend.data.into())
        }
        Err(error) => Err(error),
    }
}
pub fn detrend_mut<T: na::RealField + Copy>(
    x_values: &[T],
    y_values: &mut [T],
    polynomial_degree: usize,
) -> Result<(), &'static str> {
    let number_of_columns = polynomial_degree + 1;
    let number_of_rows = x_values.len();
    let mut a = na::DMatrix::zeros(number_of_rows, number_of_columns);

    for (row, &x) in x_values.iter().enumerate() {
        // First column is always 1
        a[(row, 0)] = T::one();

        for col in 1..number_of_columns {
            a[(row, col)] = x.powf(na::convert(col as f64));
        }
    }

    let b = na::DVector::from_row_slice(y_values);

    let decomp = na::SVD::new(a.clone(), true, true);

    match decomp.solve(&b, na::convert(1e-18f64)) {
        Ok(mat) => {
            let mut y_detrend = b - a * &mat;
            y_values.swap_with_slice(&mut y_detrend.as_mut_slice());
            Ok(())
        }
        Err(error) => Err(error),
    }
}

#[cfg(feature = "plot")]
pub fn plot_monitor<S: AsRef<std::path::Path> + std::convert::AsRef<std::ffi::OsStr>>(
    time: &[f64],
    monitor: &[Exertion],
    key: &str,
    path: S,
) {
    use plotters::prelude::*;
    let max_value = |x: &[f64]| -> f64 {
        x.iter()
            .cloned()
            .rev()
            .take(400 * 20)
            .fold(std::f64::NEG_INFINITY, f64::max)
    };
    let min_value = |x: &[f64]| -> f64 {
        x.iter()
            .cloned()
            .rev()
            .take(400 * 20)
            .fold(std::f64::INFINITY, f64::min)
    };

    let file_path = std::path::Path::new(&path).join("TOTAL_FORCES.png");
    let filename = if let Some(filename) = file_path.to_str() {
        filename.to_string()
    } else {
        eprintln!("Invalid file path: {:?}", file_path);
        return;
    };
    let plot = BitMapBackend::new(&filename, (768, 512)).into_drawing_area();
    plot.fill(&WHITE).unwrap();

    let (min_value, max_value) = {
        let force_magnitude: Option<Vec<f64>> =
            monitor.iter().map(|e| e.force.magnitude()).collect();
        (
            min_value(force_magnitude.as_ref().unwrap()),
            max_value(force_magnitude.as_ref().unwrap()),
        )
    };
    let xrange = time.last().unwrap() - time[0];
    let minmax_padding = 0.1;
    let mut chart = ChartBuilder::on(&plot)
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .margin(10)
        .build_cartesian_2d(
            -xrange * 1e-2..xrange * (1. + 1e-2),
            min_value * (1. - minmax_padding)..max_value * (1. + minmax_padding),
        )
        .unwrap();
    chart
        .configure_mesh()
        .x_desc("Time [s]")
        .y_desc(format!("{} Force [N]", key))
        .draw()
        .unwrap();

    let mut colors = colorous::TABLEAU10.iter().cycle();

    let color = colors.next().unwrap();
    let rgb = RGBColor(color.r, color.g, color.b);
    chart
        .draw_series(LineSeries::new(
            time.iter()
                .zip(monitor.iter())
                .map(|(&x, y)| (x - time[0], y.force.magnitude().unwrap())),
            &rgb,
        ))
        .unwrap()
        .label(key)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &rgb));

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE.mix(0.8))
        .position(SeriesLabelPosition::UpperRight)
        .draw()
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    /*
       use nalgebra as na;
       #[test]
       fn test_arm() {
           let force = [100f64, -33f64, 250f64];
           let force_v = na::Vector3::from_column_slice(&force);
           //let arm = na::Vector3::<f64>::new_random() * 2f64 - na::Vector3::repeat(1f64);
           let arm = na::Vector3::<f64>::from_column_slice(&[1., 1., 1.]);
           println!("ARM: {:?}", arm);
           let moment = arm.cross(&force_v);
           println!("Moment: {:?}", moment);
           let amat = na::Matrix3::new(
               0., force[2], -force[1], -force[2], 0., force[0], force[1], -force[0], 0.,
           );
           println!("A: {:#?}", amat);
           println!("Moment: {:?}", amat * arm);
           let qr = amat.svd(true, true);
           let x = qr.solve(&moment, 1e-3).unwrap();
           println!("ARM: {:?}", x);
           println!("Moment: {:?}", x.cross(&force_v));
       }
    */
    #[test]
    fn cfd_2020() {
        let monitors = MonitorsLoader::<2020>::default()
            .data_path("/fsx/Baseline2020/b2019_30z_0az_os_7ms/")
            .header_filter("Total".to_string())
            .load()
            .unwrap();
        println!(
            "Time: {:.3?}s",
            (monitors.time[0], monitors.time.last().unwrap())
        );
        println!("Force entries #: {}", monitors.forces_and_moments.len());
        monitors
            .forces_and_moments
            .keys()
            .for_each(|k| println!("Key: {}", k));
        println!(
            "Total force entries #: {}",
            monitors.total_forces_and_moments.len()
        );
    }
    /*
        #[test]
        fn load_mirror_table() {
            let mut m1 = Mirror::m1();
            m1.load(
                "/fsx/Baseline2021/Baseline2021/Baseline2021/CASES/zen00az180_OS2",
                false,
            )
            .unwrap();
            let t = m1.time().front().unwrap();
            let f: Vec<_> = m1
                .forces_and_moments()
                .filter_map(|f| f.front().map(|v| v.force.clone()))
                .collect();
            println!("{}: {:?}", t, f);
            let t = m1.time().back().unwrap();
            let f: Vec<_> = m1
                .forces_and_moments()
                .filter_map(|f| f.back().map(|v| v.force.clone()))
                .collect();
            println!("{}: {:?}", t, f);
        }
    */
    #[test]
    fn test_polyfit() {
        let (a, b) = (-1.5f64, 5f64);
        let (x, y): (Vec<_>, Vec<_>) = (0..10).map(|k| (k as f64, a * k as f64 + b)).unzip();
        let ba = polyfit(&x, &y, 1).unwrap();
        println!("ab: {:#?}", ba);
        assert!((ba[0] - b).abs() < 1e-6 && (ba[1] - a).abs() < 1e-6)
    }
    #[test]
    fn test_detrend() {
        let (a, b) = (-1.5f64, 5f64);
        let (x, y): (Vec<_>, Vec<_>) = (0..10).map(|k| (k as f64, a * k as f64 + b)).unzip();
        let ydtd = detrend(&x, &y, 1).unwrap();
        let ba = polyfit(&x, &ydtd, 1).unwrap();
        println!("ab: {:#?}", ba);
        //assert!((ba[0] - b).abs() < 1e-6 && (ba[1] - a).abs() < 1e-6)
    }
    #[test]
    fn test_detrend_mut() {
        let (a, b) = (-1.5f64, 5f64);
        let (x, mut y): (Vec<_>, Vec<_>) = (0..10).map(|k| (k as f64, a * k as f64 + b)).unzip();
        detrend_mut(&x, &mut y, 1).unwrap();
        let ba = polyfit(&x, &y, 1).unwrap();
        println!("ab: {:#?}", ba);
        //assert!((ba[0] - b).abs() < 1e-6 && (ba[1] - a).abs() < 1e-6)
    }
}
