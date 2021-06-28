use colorous;
use plotters::prelude::*;
use regex::Regex;
use std::path::Path;
use std::{collections::BTreeMap, fs::File};

#[derive(Default, Debug)]
pub struct Vector {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
}
impl Vector {
    pub fn magnitude(&self) -> Result<f64, String> {
        let (x, y, z) = (
            self.x.ok_or("x component missing")?,
            self.y.ok_or("y component missing")?,
            self.z.ok_or("z component missing")?,
        );
        Ok((x * x + y * y + z * z).sqrt())
    }
}
impl Vector {
    pub fn from_x(value: f64) -> Self {
        Self {
            x: Some(value),
            ..Default::default()
        }
    }
    pub fn from_y(value: f64) -> Self {
        Self {
            y: Some(value),
            ..Default::default()
        }
    }
    pub fn from_z(value: f64) -> Self {
        Self {
            z: Some(value),
            ..Default::default()
        }
    }
}
#[derive(Default, Debug)]
pub struct Exertion {
    pub force: Vector,
    pub moment: Vector,
}
impl Exertion {
    #[allow(dead_code)]
    pub fn from_force(force: Vector) -> Self {
        Self {
            force,
            ..Default::default()
        }
    }
    pub fn from_force_x(value: f64) -> Self {
        Self {
            force: Vector::from_x(value),
            ..Default::default()
        }
    }
    pub fn from_force_y(value: f64) -> Self {
        Self {
            force: Vector::from_y(value),
            ..Default::default()
        }
    }
    pub fn from_force_z(value: f64) -> Self {
        Self {
            force: Vector::from_z(value),
            ..Default::default()
        }
    }
    #[allow(dead_code)]
    pub fn from_moment(moment: Vector) -> Self {
        Self {
            moment,
            ..Default::default()
        }
    }
    pub fn from_moment_x(value: f64) -> Self {
        Self {
            moment: Vector::from_x(value),
            ..Default::default()
        }
    }
    pub fn from_moment_y(value: f64) -> Self {
        Self {
            moment: Vector::from_y(value),
            ..Default::default()
        }
    }
    pub fn from_moment_z(value: f64) -> Self {
        Self {
            moment: Vector::from_z(value),
            ..Default::default()
        }
    }
}
#[derive(Default, Debug)]
pub struct Monitors {
    pub time: Vec<f64>,
    pub heat_transfer_coefficients: BTreeMap<String, Vec<f64>>,
    pub forces_and_moments: BTreeMap<String, Vec<Exertion>>,
}
impl Monitors {
    pub fn len(&self) -> usize {
        self.time.len()
    }
    pub fn summary(&self) {
        let max_value = |x: &[f64]| x.iter().cloned().fold(std::f64::NEG_INFINITY, f64::max);
        let min_value = |x: &[f64]| x.iter().cloned().fold(std::f64::INFINITY, f64::min);
        let minmax = |x| (min_value(x), max_value(x));
        let stats = |x: &[f64]| {
            let n = x.len() as f64;
            let mean = x.iter().sum::<f64>() / n;
            let std = (x.iter().map(|x| x - mean).fold(0f64, |s, x| s + x * x) / n).sqrt();
            (mean, std)
        };

        println!("SUMMARY:");
        println!(" - # of records: {}", self.len());
        println!(
            " - time range: [{:8.3}-{:8.3}]s",
            self.time[0],
            self.time.last().unwrap()
        );
        let n_htc = self.heat_transfer_coefficients.len();
        if !self.heat_transfer_coefficients.is_empty() {
            println!(" - # of HTC elements: {}", n_htc);
            println!(" - HTC [W/m^2-K]:");
            println!(
                "    {:^16}: ({:^12}, {:^12})  ({:^12}, {:^12})",
                "ELEMENT", "MEAN", "STD", "MIN", "MAX"
            );
            self.heat_transfer_coefficients
                .iter()
                .for_each(|(key, value)| {
                    println!(
                        "  - {:16}: {:>12.3?}  {:>12.3?}",
                        key,
                        stats(value),
                        minmax(value)
                    );
                });
        }
        let n_fm = self.forces_and_moments.len();
        if !self.forces_and_moments.is_empty() {
            println!(" - # of elements with forces & moments: {}", n_fm);
            println!(" - Forces magnitude [N]:");
            println!(
                "    {:^16}: ({:^12}, {:^12})  ({:^12}, {:^12})",
                "ELEMENT", "MEAN", "STD", "MIN", "MAX"
            );
            self.forces_and_moments.iter().for_each(|(key, value)| {
                let force_magnitude: Result<Vec<f64>, String> =
                    value.iter().map(|e| e.force.magnitude()).collect();
                match force_magnitude {
                    Ok(value) => {
                        let force_min = min_value(&value);
                        let force_max = max_value(&value);
                        println!(
                            "  - {:16}: {:>12.3?}  {:>12.3?}",
                            key,
                            stats(&value),
                            (force_min, force_max)
                        );
                    }
                    Err(err) => println!("  - {:16}: {}", key, err),
                }
            });
            println!(" - Moments magnitude [N-m]:");
            println!(
                "    {:^16}: ({:^12}, {:^12})  ({:^12}, {:^12})",
                "ELEMENT", "MEAN", "STD", "MIN", "MAX"
            );
            self.forces_and_moments.iter().for_each(|(key, value)| {
                let moment_magnitude: Result<Vec<f64>, String> =
                    value.iter().map(|e| e.moment.magnitude()).collect();
                match moment_magnitude {
                    Ok(value) => {
                        let moment_min = min_value(&value);
                        let moment_max = max_value(&value);
                        println!(
                            "  - {:16}: {:>12.3?}  {:>12.3?}",
                            key,
                            stats(&value),
                            (moment_min, moment_max)
                        );
                    }
                    Err(err) => println!("  - {:16}: {}", key, err),
                }
            });
        }
    }
    pub fn plot_htc(&self) {
        if self.heat_transfer_coefficients.is_empty() {
            return;
        }

        let max_value =
            |x: &[f64]| -> f64 { x.iter().cloned().fold(std::f64::NEG_INFINITY, f64::max) };
        let min_value = |x: &[f64]| -> f64 { x.iter().cloned().fold(std::f64::INFINITY, f64::min) };
        let minmax = |x| (min_value(x), max_value(x));

        //let plot = BitMapBackend::new("HTC.png", (768, 512)).into_drawing_area();
        let plot = SVGBackend::new("HTC.svg", (768, 512)).into_drawing_area();
        plot.fill(&WHITE).unwrap();

        let (min_values, max_values): (Vec<_>, Vec<_>) = self
            .heat_transfer_coefficients
            .values()
            .map(|values| minmax(values))
            .unzip();
        let xrange = *self.time.last().unwrap() - self.time[0];
        let mut chart = ChartBuilder::on(&plot)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .margin(10)
            .build_cartesian_2d(
                -xrange * 1e-2..xrange * (1. + 1e-2),
                min_value(&min_values)..max_value(&max_values),
            )
            .unwrap();
        chart
            .configure_mesh()
            .x_desc("Time [s]")
            .y_desc("HTC [W/m^2-K]")
            .draw()
            .unwrap();

        let mut colors = colorous::TABLEAU10.iter().cycle();
        let mut rgbs = vec![];

        for (key, values) in self.heat_transfer_coefficients.iter() {
            let color = colors.next().unwrap();
            let rgb = RGBColor(color.r, color.g, color.b);
            rgbs.push(rgb);
            chart
                .draw_series(LineSeries::new(
                    self.time
                        .iter()
                        .zip(values.iter())
                        .map(|(&x, &y)| (x - self.time[0], y)),
                    &rgb,
                ))
                .unwrap()
                .label(key)
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));
        }
        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.8))
            .position(SeriesLabelPosition::UpperRight)
            .draw()
            .unwrap();
    }
    pub fn plot_forces(&self) {
        if self.forces_and_moments.is_empty() {
            return;
        }

        let max_value =
            |x: &[f64]| -> f64 { x.iter().cloned().fold(std::f64::NEG_INFINITY, f64::max) };
        let min_value = |x: &[f64]| -> f64 { x.iter().cloned().fold(std::f64::INFINITY, f64::min) };

        let plot = SVGBackend::new("FORCE.svg", (768, 512)).into_drawing_area();
        plot.fill(&WHITE).unwrap();

        let (min_values, max_values): (Vec<_>, Vec<_>) = self
            .forces_and_moments
            .values()
            .map(|values| {
                let force_magnitude: Result<Vec<f64>, String> =
                    values.iter().map(|e| e.force.magnitude()).collect();
                (
                    min_value(force_magnitude.as_ref().unwrap()),
                    max_value(force_magnitude.as_ref().unwrap()),
                )
            })
            .unzip();
        let xrange = *self.time.last().unwrap() - self.time[0];
        let mut chart = ChartBuilder::on(&plot)
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .margin(10)
            .build_cartesian_2d(
                -xrange * 1e-2..xrange * (1. + 1e-2),
                min_value(&min_values)..max_value(&max_values),
            )
            .unwrap();
        chart
            .configure_mesh()
            .x_desc("Time [s]")
            .y_desc("Force [N]")
            .draw()
            .unwrap();

        let mut colors = colorous::TABLEAU10.iter().cycle();

        for (key, values) in self.forces_and_moments.iter() {
            let color = colors.next().unwrap();
            let rgb = RGBColor(color.r, color.g, color.b);
            chart
                .draw_series(LineSeries::new(
                    self.time
                        .iter()
                        .zip(values.iter())
                        .map(|(&x, y)| (x - self.time[0], y.force.magnitude().unwrap())),
                    &rgb,
                ))
                .unwrap()
                .label(key)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &rgb));
        }
        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.8))
            .position(SeriesLabelPosition::UpperRight)
            .draw()
            .unwrap();
    }
    pub fn plot_moments(&self) {
        if self.forces_and_moments.is_empty() {
            return;
        }

        let max_value =
            |x: &[f64]| -> f64 { x.iter().cloned().fold(std::f64::NEG_INFINITY, f64::max) };
        let min_value = |x: &[f64]| -> f64 { x.iter().cloned().fold(std::f64::INFINITY, f64::min) };

        let plot = SVGBackend::new("MOMENTS.svg", (768, 512)).into_drawing_area();
        plot.fill(&WHITE).unwrap();

        let (min_values, max_values): (Vec<_>, Vec<_>) = self
            .forces_and_moments
            .values()
            .map(|values| {
                let force_magnitude: Result<Vec<f64>, String> =
                    values.iter().map(|e| e.moment.magnitude()).collect();
                (
                    min_value(force_magnitude.as_ref().unwrap()),
                    max_value(force_magnitude.as_ref().unwrap()),
                )
            })
            .unzip();
        let xrange = *self.time.last().unwrap() - self.time[0];
        let mut chart = ChartBuilder::on(&plot)
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .margin(10)
            .build_cartesian_2d(
                -xrange * 1e-2..xrange * (1. + 1e-2),
                min_value(&min_values)..max_value(&max_values),
            )
            .unwrap();
        chart
            .configure_mesh()
            .x_desc("Time [s]")
            .y_desc("Moment [N-m]")
            .draw()
            .unwrap();

        let mut colors = colorous::TABLEAU10.iter().cycle();

        for (key, values) in self.forces_and_moments.iter() {
            let color = colors.next().unwrap();
            let rgb = RGBColor(color.r, color.g, color.b);
            chart
                .draw_series(LineSeries::new(
                    self.time
                        .iter()
                        .zip(values.iter())
                        .map(|(&x, y)| (x - self.time[0], y.moment.magnitude().unwrap())),
                    &rgb,
                ))
                .unwrap()
                .label(key)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &rgb));
        }
        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.8))
            .position(SeriesLabelPosition::UpperRight)
            .draw()
            .unwrap();
    }
}
pub struct MonitorsLoader {
    path: String,
    time_range: (f64, f64),
    header_regex: String,
}
impl Default for MonitorsLoader {
    fn default() -> Self {
        Self {
            path: String::from("monitors.csv"),
            time_range: (0f64, f64::INFINITY),
            header_regex: String::from(r"\w+"),
        }
    }
}
impl MonitorsLoader {
    pub fn data_path(self, data_path: String) -> Self {
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
    pub fn load(self) -> Result<Monitors, Box<dyn std::error::Error>> {
        let csv_file = File::open(self.path)?;
        let mut rdr = csv::Reader::from_reader(csv_file);
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

        let mut monitors = Monitors::default();

        for result in rdr.records() {
            let record = result?;
            let time = record.iter().next().unwrap().parse::<f64>()?;
            if time < self.time_range.0 || time > self.time_range.1 {
                continue;
            };
            monitors.time.push(time);
            for (data, header) in record
                .iter()
                .skip(1)
                .zip(headers.iter().skip(1))
                .filter(|(_, h)| re_header.is_match(h))
            {
                // HTC
                if let Some(capts) = re_htc.captures(header) {
                    let key = capts.get(1).unwrap().as_str().to_owned();
                    let value = data.parse::<f64>()?;
                    monitors
                        .heat_transfer_coefficients
                        .entry(key)
                        .or_insert_with(Vec::new)
                        .push(value);
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
        Ok(monitors)
    }
}
