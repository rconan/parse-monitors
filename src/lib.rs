use bzip2::bufread::BzDecoder;
use colorous;
use log;
use plotters::prelude::*;
use regex::Regex;
use serde_pickle as pkl;
use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    fs::File,
    io::{BufReader, Read},
    ops::{Add, AddAssign, Deref, DerefMut, Div, Sub},
    path::Path,
    time::Instant,
};
use windloading::{Loads, WindLoads};

pub struct FemNodes(BTreeMap<String, Vector>);
impl Deref for FemNodes {
    type Target = BTreeMap<String, Vector>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for FemNodes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Default for FemNodes {
    fn default() -> Self {
        let mut fem = FemNodes(BTreeMap::new());
        fem.insert("M1cov1".to_string(), [0., 13.5620, 5.3064].into());
        fem.insert("M1cov2".to_string(), [11.7451, 6.7810, 5.3064].into());
        fem.insert("M1cov3".to_string(), [11.7451, -6.7810, 5.3064].into());
        fem.insert("M1cov4".to_string(), [0., -13.5621, 5.3064].into());
        fem.insert("M1cov5".to_string(), [-11.7451, -6.7810, 5.3064].into());
        fem.insert("M1cov6".to_string(), [-11.7451, 6.7810, 5.3064].into());
        fem.insert("M1covin1".to_string(), [2.3650, 4.0963, 4.7000].into());
        fem.insert("M1covin2".to_string(), [4.3000, 0., 4.7000].into());
        fem.insert("M1covin3".to_string(), [2.3650, -4.0963, 4.7000].into());
        fem.insert("M1covin4".to_string(), [-2.3650, -4.0963, 4.7000].into());
        fem.insert("M1covin5".to_string(), [-4.3000, 0., 4.7000].into());
        fem.insert("M1covin6".to_string(), [-2.3650, 4.0963, 4.7000].into());
        fem
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vector {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
}
impl Vector {
    pub fn zero() -> Self {
        Self {
            x: Some(0f64),
            y: Some(0f64),
            z: Some(0f64),
        }
    }
    pub fn magnitude(&self) -> Option<f64> {
        if let Some((x, y, z)) = self.as_tuple() {
            Some((x * x + y * y + z * z).sqrt())
        } else {
            None
        }
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
    pub fn as_tuple(&self) -> Option<(&f64, &f64, &f64)> {
        match self {
            Vector {
                x: Some(a1),
                y: Some(a2),
                z: Some(a3),
            } => Some((a1, a2, a3)),
            _ => None,
        }
    }
    pub fn into_tuple(self) -> Option<(f64, f64, f64)> {
        match self {
            Vector {
                x: Some(a1),
                y: Some(a2),
                z: Some(a3),
            } => Some((a1, a2, a3)),
            _ => None,
        }
    }
    pub fn as_array(&self) -> [&f64; 3] {
        match self {
            Vector {
                x: Some(a1),
                y: Some(a2),
                z: Some(a3),
            } => Ok([a1, a2, a3]),
            _ => Err(""),
        }
        .unwrap()
    }
    pub fn cross(&self, other: &Vector) -> Option<Vector> {
        if let (Some((a1, a2, a3)), Some((b1, b2, b3))) = (self.as_tuple(), other.as_tuple()) {
            Some(Vector {
                x: Some(a2 * b3 - a3 * b2),
                y: Some(a3 * b1 - a1 * b3),
                z: Some(a1 * b2 - a2 * b1),
            })
        } else {
            None
        }
    }
    pub fn norm_squared(&self) -> Option<f64> {
        if let Some((a1, a2, a3)) = self.as_tuple() {
            Some(a1 * a1 + a2 * a2 + a3 * a3)
        } else {
            None
        }
    }
}
impl Sub for &Vector {
    type Output = Option<Vector>;

    fn sub(self, rhs: Self) -> Self::Output {
        if let (Some((a1, a2, a3)), Some((b1, b2, b3))) = (self.as_tuple(), rhs.as_tuple()) {
            Some(Vector {
                x: Some(a1 - b1),
                y: Some(a2 - b2),
                z: Some(a3 - b3),
            })
        } else {
            None
        }
    }
}
impl Add for &Vector {
    type Output = Option<Vector>;

    fn add(self, rhs: Self) -> Self::Output {
        if let (Some((a1, a2, a3)), Some((b1, b2, b3))) = (self.as_tuple(), rhs.as_tuple()) {
            Some(Vector {
                x: Some(a1 + b1),
                y: Some(a2 + b2),
                z: Some(a3 + b3),
            })
        } else {
            None
        }
    }
}
impl AddAssign<&Vector> for &mut Vector {
    fn add_assign(&mut self, other: &Vector) {
        if let (Some((a1, a2, a3)), Some((b1, b2, b3))) = (self.as_tuple(), other.as_tuple()) {
            **self = Vector {
                x: Some(a1 + b1),
                y: Some(a2 + b2),
                z: Some(a3 + b3),
            }
        }
    }
}
impl Div<f64> for Vector {
    type Output = Option<Self>;

    fn div(self, rhs: f64) -> Self::Output {
        if let Some((a1, a2, a3)) = self.as_tuple() {
            Some(Vector {
                x: Some(a1 / rhs),
                y: Some(a2 / rhs),
                z: Some(a3 / rhs),
            })
        } else {
            None
        }
    }
}
impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:6.3?},{:6.3?},{:6.3?}]", self.x, self.y, self.z)
    }
}
impl From<[f64; 3]> for Vector {
    fn from(v: [f64; 3]) -> Self {
        Vector {
            x: Some(v[0]),
            y: Some(v[1]),
            z: Some(v[2]),
        }
    }
}
#[derive(Default, Debug, Clone)]
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
    pub fn into_local(&mut self, node: &Vector) -> &mut Self {
        if let Some(v) = node.cross(&self.force) {
            self.moment = (&self.moment - &v).unwrap();
        }
        self
    }
}
#[derive(Default, Debug)]
pub struct Monitors {
    pub time: Vec<f64>,
    pub heat_transfer_coefficients: BTreeMap<String, Vec<f64>>,
    pub forces_and_moments: BTreeMap<String, Vec<Exertion>>,
    pub total_forces_and_moments: Vec<Exertion>,
}
impl Monitors {
    pub fn len(&self) -> usize {
        self.time.len()
    }
    pub fn into_local(&mut self) -> &mut Self {
        let nodes = FemNodes::default();
        for (key, value) in self.forces_and_moments.iter_mut() {
            if let Some(node) = nodes.get(key) {
                value.iter_mut().for_each(|v| {
                    (*v).into_local(node);
                });
            }
        }
        self
    }
    pub fn summary(&mut self) {
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
            let (total_force, total_moment): (Vec<_>, Vec<_>) =
                self.forces_and_moments.values().fold(
                    (
                        vec![Vector::zero(); self.len()],
                        vec![Vector::zero(); self.len()],
                    ),
                    |(mut fa, mut ma), value| {
                        fa.iter_mut()
                            .zip(value.iter())
                            .for_each(|(mut fa, e)| fa += &e.force);
                        ma.iter_mut()
                            .zip(value.iter())
                            .for_each(|(mut ma, e)| ma += &e.moment);
                        (fa, ma)
                    },
                );
            let total_force_magnitude: Option<Vec<f64>> =
                total_force.iter().map(|x| x.magnitude()).collect();
            let total_moment_magnitude: Option<Vec<f64>> =
                total_moment.iter().map(|x| x.magnitude()).collect();
            println!(" - Forces magnitude [N]:");
            println!(
                "    {:^16}: ({:^12}, {:^12})  ({:^12}, {:^12})",
                "ELEMENT", "MEAN", "STD", "MIN", "MAX"
            );
            self.forces_and_moments.iter().for_each(|(key, value)| {
                let force_magnitude: Option<Vec<f64>> =
                    value.iter().map(|e| e.force.magnitude()).collect();
                Self::display(key, force_magnitude);
            });
            Self::display("TOTAL", total_force_magnitude);
            println!(" - Moments magnitude [N-m]:");
            println!(
                "    {:^16}: ({:^12}, {:^12})  ({:^12}, {:^12})",
                "ELEMENT", "MEAN", "STD", "MIN", "MAX"
            );
            self.forces_and_moments.iter().for_each(|(key, value)| {
                let moment_magnitude: Option<Vec<f64>> =
                    value.iter().map(|e| e.moment.magnitude()).collect();
                Self::display(key, moment_magnitude);
            });
            Self::display("TOTAL", total_moment_magnitude);
            self.total_forces_and_moments = total_force
                .into_iter()
                .zip(total_moment.into_iter())
                .map(|(force, moment)| Exertion { force, moment })
                .collect();
        }
    }
    pub fn total_exertion(&mut self) -> &mut Self {
        let (total_force, total_moment): (Vec<_>, Vec<_>) = self.forces_and_moments.values().fold(
            (
                vec![Vector::zero(); self.len()],
                vec![Vector::zero(); self.len()],
            ),
            |(mut fa, mut ma), value| {
                fa.iter_mut()
                    .zip(value.iter())
                    .for_each(|(mut fa, e)| fa += &e.force);
                ma.iter_mut()
                    .zip(value.iter())
                    .for_each(|(mut ma, e)| ma += &e.moment);
                (fa, ma)
            },
        );
        self.total_forces_and_moments = total_force
            .into_iter()
            .zip(total_moment.into_iter())
            .map(|(force, moment)| Exertion { force, moment })
            .collect();
        self
    }
    pub fn display(key: &str, data: Option<Vec<f64>>) {
        let max_value = |x: &[f64]| x.iter().cloned().fold(std::f64::NEG_INFINITY, f64::max);
        let min_value = |x: &[f64]| x.iter().cloned().fold(std::f64::INFINITY, f64::min);
        let stats = |x: &[f64]| {
            let n = x.len() as f64;
            let mean = x.iter().sum::<f64>() / n;
            let std = (x.iter().map(|x| x - mean).fold(0f64, |s, x| s + x * x) / n).sqrt();
            (mean, std)
        };
        match data {
            Some(value) => {
                let data_min = min_value(&value);
                let data_max = max_value(&value);
                println!(
                    "  - {:16}: {:>12.3?}  {:>12.3?}",
                    key,
                    stats(&value),
                    (data_min, data_max)
                );
            }
            None => println!("  - {:16}: {:?}", key, None::<f64>),
        }
    }
    pub fn to_csv(&self, filename: String) -> Result<Vec<()>, csv::Error> {
        let mut wtr = csv::Writer::from_path(filename)?;
        let mut keys = vec![String::from("Time [s]")];
        keys.extend(
            self.forces_and_moments
                .keys()
                .filter(|&k| k.as_str() != "M1covin2")
                .flat_map(|k| {
                    vec![
                        format!("{} X force [N]", k),
                        format!("{} Y force [N]", k),
                        format!("{} Z force [N]", k),
                        format!("{} X moment [N-m]", k),
                        format!("{} Y moment [N-m]", k),
                        format!("{} Z moment [N-m]", k),
                    ]
                }),
        );
        wtr.write_record(&keys)?;
        self.time
            .iter()
            .enumerate()
            .map(|(k, t)| {
                let mut record = vec![format!("{}", t)];
                record.extend(
                    self.forces_and_moments
                        .iter()
                        .filter(|(k, _)| k.as_str() != "M1covin2")
                        .flat_map(|(_, v)| {
                            vec![
                                format!("{}", v[k].force.x.unwrap()),
                                format!("{}", v[k].force.y.unwrap()),
                                format!("{}", v[k].force.z.unwrap()),
                                format!("{}", v[k].moment.x.unwrap()),
                                format!("{}", v[k].moment.y.unwrap()),
                                format!("{}", v[k].moment.z.unwrap()),
                            ]
                        }),
                );
                wtr.write_record(&record)
            })
            .collect::<Result<Vec<()>, csv::Error>>()
    }
    pub fn m1covers_windloads(&self) -> Result<(), Box<dyn Error>> {
        let keys = vec![
            "M1cov1", "M1cov6", "M1cov5", "M1cov4", "M1cov3", "M1cov2", "M1covin2", "M1covin1",
            "M1covin6", "M1covin5", "M1covin4", "M1covin3",
        ];
        let mut loads: Vec<Vec<f64>> = Vec::with_capacity(72 * self.len());
        for k in 0..self.len() {
            let mut fm: Vec<f64> = Vec::with_capacity(72);
            for &key in &keys {
                let exrt = self.forces_and_moments.get(key).unwrap().get(k).unwrap();
                fm.append(
                    &mut exrt
                        .force
                        .as_array()
                        .iter()
                        .cloned()
                        .chain(exrt.moment.as_array().iter().cloned())
                        .cloned()
                        .collect::<Vec<f64>>(),
                );
            }
            loads.push(fm);
        }
        let mut windloads: WindLoads = Default::default();
        windloads.time = self.time.clone();
        windloads.loads = vec![Some(Loads::OSSMirrorCovers6F(loads))];
        let mut file = File::create("windloads.pkl")?;
        pkl::to_writer(&mut file, &windloads, true)?;
        Ok(())
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

        let plot = SVGBackend::new("FORCE.svg", (768, 512)).into_drawing_area();
        plot.fill(&WHITE).unwrap();

        let (min_values, max_values): (Vec<_>, Vec<_>) = self
            .forces_and_moments
            .values()
            .map(|values| {
                let force_magnitude: Option<Vec<f64>> =
                    values.iter().map(|e| e.force.magnitude()).collect();
                (
                    min_value(force_magnitude.as_ref().unwrap()),
                    max_value(force_magnitude.as_ref().unwrap()),
                )
            })
            .unzip();
        let xrange = *self.time.last().unwrap() - self.time[0];
        let minmax_padding = 0.1;
        let mut chart = ChartBuilder::on(&plot)
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .margin(10)
            .build_cartesian_2d(
                -xrange * 1e-2..xrange * (1. + 1e-2),
                min_value(&min_values) * (1. - minmax_padding)
                    ..max_value(&max_values) * (1. + minmax_padding),
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
                let force_magnitude: Option<Vec<f64>> =
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
    header_exclude_regex: Option<String>,
}
impl Default for MonitorsLoader {
    fn default() -> Self {
        Self {
            path: String::from("monitors.csv"),
            time_range: (0f64, f64::INFINITY),
            header_regex: String::from(r"\w+"),
            header_exclude_regex: None,
        }
    }
}
impl MonitorsLoader {
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

/*
#[cfg(test)]
mod tests {
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
}
*/
