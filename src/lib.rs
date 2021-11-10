use bzip2::bufread::BzDecoder;
use plotters::prelude::*;
use regex::Regex;
use std::{
    collections::{BTreeMap, VecDeque},
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
pub mod pressure;
pub mod report;

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
/// Mirror type
#[derive(Debug)]
pub enum Mirror {
    M1 {
        time: VecDeque<f64>,
        force: BTreeMap<String, VecDeque<Exertion>>,
    },
    M2 {
        time: VecDeque<f64>,
        force: BTreeMap<String, VecDeque<Exertion>>,
    },
}
impl Mirror {
    pub fn m1() -> Self {
        let mut force: BTreeMap<String, VecDeque<Exertion>> = BTreeMap::new();
        (1..=7).for_each(|k| {
            force.entry(format!("S{}", k)).or_default();
        });
        Mirror::M1 {
            time: VecDeque::new(),
            force,
        }
    }
    pub fn m2() -> Self {
        let mut force: BTreeMap<String, VecDeque<Exertion>> = BTreeMap::new();
        (1..=7).for_each(|k| {
            force.entry(format!("S{}", k)).or_default();
        });
        Mirror::M2 {
            time: VecDeque::new(),
            force,
        }
    }
    pub fn summary(&self) {
        let (mirror, time, force) = match self {
            Mirror::M1 { time, force } => ("M1", time, force),
            Mirror::M2 { time, force } => ("M2", time, force),
        };
        println!("{} SUMMARY:", mirror);
        println!(" - # of records: {}", time.len());
        println!(
            " - time range: [{:8.3}-{:8.3}]s",
            time.front().unwrap(),
            time.back().unwrap()
        );
        println!(
            "    {:^16}: ({:^12}, {:^12})  ({:^12}, {:^12})",
            "ELEMENT", "MEAN", "STD", "MIN", "MAX"
        );
        for (key, value) in force.iter() {
            let force_magnitude: Option<Vec<f64>> =
                value.iter().map(|e| e.force.magnitude()).collect();
            Monitors::display(key, force_magnitude);
        }
    }
    pub fn load<P: AsRef<Path>>(
        &mut self,
        path: P,
        net_force: bool,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let (filename, time, force) = match self {
            Mirror::M1 { time, force } => ("M1_segments_force.csv", time, force),
            Mirror::M2 { time, force } => ("M2_segments_force.csv", time, force),
        };
        let path = Path::new(path.as_ref());
        if let Ok(csv_file) = File::open(&path.join(filename)) {
            let mut rdr = csv::Reader::from_reader(csv_file);
            for result in rdr.records() {
                let record = result?;
                let mut record_iter = record.iter();
                let t = record_iter.next().unwrap().parse::<f64>()?;
                if let Some(t_b) = time.back() {
                    if t >= *t_b {
                        time.push_back(t);
                        for fm in force.values_mut() {
                            let f: Vector = [
                                record_iter.next().unwrap().parse::<f64>()?,
                                record_iter.next().unwrap().parse::<f64>()?,
                                record_iter.next().unwrap().parse::<f64>()?,
                            ]
                            .into();
                            fm.push_back(Exertion::from_force(f))
                        }
                    } else {
                        if let Some(index) = time.iter().rposition(|&x| x < t) {
                            time.insert(index + 1, t);
                            for fm in force.values_mut() {
                                let f: Vector = [
                                    record_iter.next().unwrap().parse::<f64>()?,
                                    record_iter.next().unwrap().parse::<f64>()?,
                                    record_iter.next().unwrap().parse::<f64>()?,
                                ]
                                .into();
                                fm.insert(index + 1, Exertion::from_force(f))
                            }
                        } else {
                            time.push_front(t);
                            for fm in force.values_mut() {
                                let f: Vector = [
                                    record_iter.next().unwrap().parse::<f64>()?,
                                    record_iter.next().unwrap().parse::<f64>()?,
                                    record_iter.next().unwrap().parse::<f64>()?,
                                ]
                                .into();
                                fm.push_front(Exertion::from_force(f))
                            }
                        }
                    }
                } else {
                    time.push_back(t);
                    for fm in force.values_mut() {
                        let f: Vector = [
                            record_iter.next().unwrap().parse::<f64>()?,
                            record_iter.next().unwrap().parse::<f64>()?,
                            record_iter.next().unwrap().parse::<f64>()?,
                        ]
                        .into();
                        fm.push_back(Exertion::from_force(f))
                    }
                }
            }
            if net_force {
                if let Mirror::M1 { time, force } = self {
                    let ts = *time.front().unwrap();
                    let te = *time.back().unwrap();
                    let monitors = MonitorsLoader::<2021>::default()
                        .data_path(path)
                        .header_filter("M1cell".to_string())
                        .start_time(ts)
                        .end_time(te)
                        .load()?;
                    let m1_cell_force: Vec<_> = monitors.forces_and_moments["M1cell"]
                        .iter()
                        .map(|x| x.force.clone())
                        .collect();
                    assert_eq!(
                        time.len(),
                        m1_cell_force.len(),
                        "M1 segments and M1 cell # of sample do not match"
                    );
                    for v in force.values_mut() {
                        for (e, cell) in v.iter_mut().zip(&m1_cell_force) {
                            let mut f = &mut e.force;
                            f += &(cell / 7f64).unwrap();
                        }
                    }
                }
            }
            Ok(self)
        } else {
            Err(format!("Cannot open {:?}", &path).into())
        }
    }
    pub fn time(&self) -> &VecDeque<f64> {
        match self {
            Mirror::M1 { time, .. } => time,
            Mirror::M2 { time, .. } => time,
        }
    }
    pub fn force(&self) -> impl Iterator<Item = &VecDeque<Exertion>> {
        match self {
            Mirror::M1 { force, .. } => force.values(),
            Mirror::M2 { force, .. } => force.values(),
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
    #[test]
    fn load_mirror_table() {
        let mut m1 = Mirror::m1();
        m1.load("/fsx/Baseline2021/Baseline2021/Baseline2021/CASES/zen00az180_OS2")
            .unwrap();
        let t = m1.time().front().unwrap();
        let f: Vec<_> = m1
            .force()
            .filter_map(|f| f.front().map(|v| v.force.clone()))
            .collect();
        println!("{}: {:?}", t, f);
        let t = m1.time().back().unwrap();
        let f: Vec<_> = m1
            .force()
            .filter_map(|f| f.back().map(|v| v.force.clone()))
            .collect();
        println!("{}: {:?}", t, f);
    }
}
