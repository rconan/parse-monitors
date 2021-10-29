use bzip2::bufread::BzDecoder;
use plotters::prelude::*;
use regex::Regex;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
    time::Instant,
};

mod vector;
pub use vector::Vector;
mod monitors;
pub use monitors::{Exertion, Monitors};
pub mod cfd;
pub mod domeseeing;
pub use domeseeing::{Band, DomeSeeing};
pub use pressure;

pub struct MonitorsLoader<const YEAR: u32> {
    path: String,
    time_range: (f64, f64),
    header_regex: String,
    header_exclude_regex: Option<String>,
}
impl<const YEAR: u32> Default for MonitorsLoader<YEAR> {
    fn default() -> Self {
        Self {
            path: String::from("monitors.csv"),
            time_range: (0f64, f64::INFINITY),
            header_regex: String::from(r"\w+"),
            header_exclude_regex: None,
        }
    }
}
impl<const YEAR: u32> MonitorsLoader<YEAR> {
    pub fn data_path<S: AsRef<Path> + std::convert::AsRef<std::ffi::OsStr>>(
        self,
        data_path: S,
    ) -> Self {
        let path = Path::new(&data_path).join("monitors.csv");
        Self {
            path: path.to_str().unwrap().to_owned(),
            ..self
        }
    }
    pub fn start_time(self, time: f64) -> Self {
        Self {
            time_range: (time, self.time_range.1),
            ..self
        }
    }
    pub fn end_time(self, time: f64) -> Self {
        Self {
            time_range: (self.time_range.0, time),
            ..self
        }
    }
    pub fn header_filter(self, header_regex: String) -> Self {
        Self {
            header_regex,
            ..self
        }
    }
    pub fn exclude_filter<S: Into<String>>(self, header_exclude_regex: S) -> Self {
        Self {
            header_exclude_regex: Some(header_exclude_regex.into()),
            ..self
        }
    }
}
impl MonitorsLoader<2021> {
    pub fn load(self) -> Result<Monitors, Box<dyn std::error::Error>> {
        let csv_file = File::open(Path::new(&self.path).with_extension("csv.bz2"))?;
        log::info!("Loading {:?}...", csv_file);
        let now = Instant::now();
        let buf = BufReader::new(csv_file);
        let mut bz2 = BzDecoder::new(buf);
        let mut contents = String::new();
        bz2.read_to_string(&mut contents)?;
        let mut rdr = csv::Reader::from_reader(contents.as_bytes());

        let headers: Vec<_> = {
            let headers = rdr.headers()?;
            //headers.iter().take(20).for_each(|h| println!("{}", h));
            headers.into_iter().map(|h| h.to_string()).collect()
        };

        let re_htc = Regex::new(
            r"(\w+) Monitor: Surface Average of Heat Transfer Coefficient \(W/m\^2-K\)",
        )?;
        let re_force = Regex::new(r"(\w+)_([XYZ]) Monitor: Force \(N\)")?;
        let re_moment = Regex::new(r"(\w+)Mom_([XYZ]) Monitor: Moment \(N-m\)")?;

        let re_header = Regex::new(&self.header_regex)?;
        let re_x_header = if let Some(re) = self.header_exclude_regex {
            Some(Regex::new(&re)?)
        } else {
            None
        };

        let mut monitors = Monitors::default();

        for result in rdr.records() {
            let record = result?;
            let time = record.iter().next().unwrap().parse::<f64>()?;
            if time < self.time_range.0 || time > self.time_range.1 {
                continue;
            };
            monitors.time.push(time);
            for (data, header) in record.iter().skip(1).zip(headers.iter().skip(1)).filter(
                |(_, h)| match &re_x_header {
                    Some(re_x_header) => re_header.is_match(h) && !re_x_header.is_match(h),
                    None => re_header.is_match(h),
                },
            ) {
                // HTC
                if let Some(capts) = re_htc.captures(header) {
                    let key = capts.get(1).unwrap().as_str().to_owned();
                    let value = data.parse::<f64>()?;
                    monitors
                        .heat_transfer_coefficients
                        .entry(key)
                        .or_insert_with(Vec::new)
                        .push(value.abs());
                }
                // FORCE
                if let Some(capts) = re_force.captures(header) {
                    let key = capts.get(1).unwrap().as_str().to_owned();
                    let value = data.parse::<f64>()?;
                    let exertions = monitors
                        .forces_and_moments
                        .entry(key)
                        .or_insert(vec![Exertion::default()]);
                    let exertion = exertions.last_mut().unwrap();
                    match capts.get(2).unwrap().as_str() {
                        "X" => match exertion.force.x {
                            Some(_) => exertions.push(Exertion::from_force_x(value)),
                            None => {
                                exertion.force.x = Some(value);
                            }
                        },
                        "Y" => match exertion.force.y {
                            Some(_) => exertions.push(Exertion::from_force_y(value)),
                            None => {
                                exertion.force.y = Some(value);
                            }
                        },
                        "Z" => match exertion.force.z {
                            Some(_) => exertions.push(Exertion::from_force_z(value)),
                            None => {
                                exertion.force.z = Some(value);
                            }
                        },
                        &_ => (),
                    };
                }
                // MOMENT
                if let Some(capts) = re_moment.captures(header) {
                    let key = capts.get(1).unwrap().as_str().to_owned();
                    let value = data.parse::<f64>()?;
                    let exertions = monitors
                        .forces_and_moments
                        .entry(key)
                        .or_insert(vec![Exertion::default()]);
                    let exertion = exertions.last_mut().unwrap();
                    match capts.get(2).unwrap().as_str() {
                        "X" => match exertion.moment.x {
                            Some(_) => exertions.push(Exertion::from_moment_x(value)),
                            None => {
                                exertion.moment.x = Some(value);
                            }
                        },
                        "Y" => match exertion.moment.y {
                            Some(_) => exertions.push(Exertion::from_moment_y(value)),
                            None => {
                                exertion.moment.y = Some(value);
                            }
                        },
                        "Z" => match exertion.moment.z {
                            Some(_) => exertions.push(Exertion::from_moment_z(value)),
                            None => {
                                exertion.moment.z = Some(value);
                            }
                        },
                        &_ => (),
                    };
                }
            }
        }
        log::info!("... loaded in {:}s", now.elapsed().as_secs());
        Ok(monitors)
    }
}
impl MonitorsLoader<2020> {
    pub fn load(self) -> Result<Monitors, Box<dyn std::error::Error>> {
        let csv_file = File::open(Path::new(&self.path).with_file_name("FORCES.txt"))?;
        log::info!("Loading {:?}...", csv_file);
        let now = Instant::now();
        let buf = BufReader::new(csv_file);
        let mut rdr = csv::Reader::from_reader(buf);

        let headers: Vec<_> = {
            let headers = rdr.headers()?;
            headers.into_iter().map(|h| h.to_string()).collect()
        };

        let re_force = Regex::new(r"(\w+) ([xyz]) Monitor: Force \(N\)")?;
        //        let re_moment = Regex::new(r"(\w+)Mom_([XYZ]) Monitor: Moment \(N-m\)")?;

        let re_header = Regex::new(&self.header_regex)?;
        let re_x_header = if let Some(re) = self.header_exclude_regex {
            Some(Regex::new(&re)?)
        } else {
            None
        };

        let mut monitors = Monitors::default();

        for result in rdr.records() {
            let record = result?;
            let time = record.iter().next().unwrap().parse::<f64>()?;
            if time < self.time_range.0 || time > self.time_range.1 {
                continue;
            };
            monitors.time.push(time);
            for (data, header) in record.iter().skip(1).zip(headers.iter().skip(1)).filter(
                |(_, h)| match &re_x_header {
                    Some(re_x_header) => re_header.is_match(h) && !re_x_header.is_match(h),
                    None => re_header.is_match(h),
                },
            ) {
                // FORCE
                if let Some(capts) = re_force.captures(header) {
                    let key = capts.get(1).unwrap().as_str().to_owned();
                    let value = data.parse::<f64>()?;
                    let exertions = monitors
                        .forces_and_moments
                        .entry(key)
                        .or_insert(vec![Exertion::default()]);
                    let exertion = exertions.last_mut().unwrap();
                    match capts.get(2).unwrap().as_str() {
                        "x" => match exertion.force.x {
                            Some(_) => exertions.push(Exertion::from_force_x(value)),
                            None => {
                                exertion.force.x = Some(value);
                            }
                        },
                        "y" => match exertion.force.y {
                            Some(_) => exertions.push(Exertion::from_force_y(value)),
                            None => {
                                exertion.force.y = Some(value);
                            }
                        },
                        "z" => match exertion.force.z {
                            Some(_) => exertions.push(Exertion::from_force_z(value)),
                            None => {
                                exertion.force.z = Some(value);
                            }
                        },
                        &_ => (),
                    };
                }
                /*
                                // MOMENT
                                if let Some(capts) = re_moment.captures(header) {
                                    let key = capts.get(1).unwrap().as_str().to_owned();
                                    let value = data.parse::<f64>()?;
                                    let exertions = monitors
                                        .forces_and_moments
                                        .entry(key)
                                        .or_insert(vec![Exertion::default()]);
                                    let exertion = exertions.last_mut().unwrap();
                                    match capts.get(2).unwrap().as_str() {
                                        "X" => match exertion.moment.x {
                                            Some(_) => exertions.push(Exertion::from_moment_x(value)),
                                            None => {
                                                exertion.moment.x = Some(value);
                                            }
                                        },
                                        "Y" => match exertion.moment.y {
                                            Some(_) => exertions.push(Exertion::from_moment_y(value)),
                                            None => {
                                                exertion.moment.y = Some(value);
                                            }
                                        },
                                        "Z" => match exertion.moment.z {
                                            Some(_) => exertions.push(Exertion::from_moment_z(value)),
                                            None => {
                                                exertion.moment.z = Some(value);
                                            }
                                        },
                                        &_ => (),
                                    };
                                }
                */
            }
        }
        if let Some(data) = monitors.forces_and_moments.remove("Total") {
            monitors.total_forces_and_moments = data;
        } else {
            return Err("No Total entry found".into());
        }
        log::info!("... loaded in {:}s", now.elapsed().as_secs());
        Ok(monitors)
    }
}

pub fn plot_monitor<S: AsRef<Path> + std::convert::AsRef<std::ffi::OsStr>>(
    time: &[f64],
    monitor: &[Exertion],
    key: &str,
    path: S,
) {
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

    let file_path = Path::new(&path).join("TOTAL_FORCES.png");
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
}
