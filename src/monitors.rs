use crate::Vector;
use plotters::prelude::*;
use serde_pickle as pkl;
use std::{
    collections::BTreeMap,
    error::Error,
    fs::File,
    ops::{Deref, DerefMut},
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
/// A force ['Vector'] and a moment ['Vector']
#[derive(Default, Debug, Clone)]
pub struct Exertion {
    pub force: Vector,
    pub moment: Vector,
}
impl Exertion {
    /// Build from a force ['Vector']
    #[allow(dead_code)]
    pub fn from_force(force: Vector) -> Self {
        Self {
            force,
            ..Default::default()
        }
    }
    /// Build from a force x ['Vector'] component
    pub fn from_force_x(value: f64) -> Self {
        Self {
            force: Vector::from_x(value),
            ..Default::default()
        }
    }
    /// Build from a force y ['Vector'] component
    pub fn from_force_y(value: f64) -> Self {
        Self {
            force: Vector::from_y(value),
            ..Default::default()
        }
    }
    /// Build from a force z ['Vector'] component
    pub fn from_force_z(value: f64) -> Self {
        Self {
            force: Vector::from_z(value),
            ..Default::default()
        }
    }
    /// Build from a moment ['Vector']
    #[allow(dead_code)]
    pub fn from_moment(moment: Vector) -> Self {
        Self {
            moment,
            ..Default::default()
        }
    }
    /// Build from a moment x ['Vector'] component
    pub fn from_moment_x(value: f64) -> Self {
        Self {
            moment: Vector::from_x(value),
            ..Default::default()
        }
    }
    /// Build from a moment y ['Vector'] component
    pub fn from_moment_y(value: f64) -> Self {
        Self {
            moment: Vector::from_y(value),
            ..Default::default()
        }
    }
    /// Build from a moment z ['Vector'] component
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
/// Gather all the monitors of a CFD run
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
    /// Return a latex table with HTC monitors summary
    pub fn htc_latex_table(&self, stats_duration: f64) -> Option<String> {
        let max_value = |x: &[f64]| x.iter().cloned().fold(std::f64::NEG_INFINITY, f64::max);
        let min_value = |x: &[f64]| x.iter().cloned().fold(std::f64::INFINITY, f64::min);
        let minmax = |x: &[f64]| (min_value(x), max_value(x));
        let stats = |x: &[f64]| {
            let n = x.len() as f64;
            let mean = x.iter().sum::<f64>() / n;
            let std = (x.iter().map(|x| x - mean).fold(0f64, |s, x| s + x * x) / n).sqrt();
            (mean, std)
        };
        if self.heat_transfer_coefficients.is_empty() {
            None
        } else {
            let duration = self.time.last().unwrap();
            let time_filter: Vec<_> = self
                .time
                .iter()
                .map(|t| t - duration + stats_duration >= 0f64)
                .collect();
            let data: Vec<_> = self
                .heat_transfer_coefficients
                .iter()
                .map(|(key, value)| {
                    let time_filtered_value: Vec<_> = value
                        .iter()
                        .zip(time_filter.iter())
                        .filter(|(_, t)| **t)
                        .map(|(v, _)| *v)
                        .collect();
                    let (mean, std) = stats(&time_filtered_value);
                    let (min, max) = minmax(&time_filtered_value);
                    format!(
                        " {:} & {:.3} & {:.3} & {:.3} & {:.3} \\\\",
                        key, mean, std, min, max
                    )
                })
                .collect();
            Some(format!(
                r#"
\begin{{tabular}}{{crrrr}}\toprule
 ELEMENT & MEAN & STD & MIN & MAX \\\hline
{}
\bottomrule
\end{{tabular}}
"#,
                data.join("\n")
            ))
        }
    }
    /// Return a latex table with force monitors summary
    pub fn force_latex_table(&self, stats_duration: f64) -> Option<String> {
        let max_value = |x: &[f64]| x.iter().cloned().fold(std::f64::NEG_INFINITY, f64::max);
        let min_value = |x: &[f64]| x.iter().cloned().fold(std::f64::INFINITY, f64::min);
        let minmax = |x: &[f64]| (min_value(x), max_value(x));
        let stats = |x: &[f64]| {
            let n = x.len() as f64;
            let mean = x.iter().sum::<f64>() / n;
            let std = (x.iter().map(|x| x - mean).fold(0f64, |s, x| s + x * x) / n).sqrt();
            (mean, std)
        };
        if self.forces_and_moments.is_empty() {
            None
        } else {
            let duration = self.time.last().unwrap();
            let time_filter: Vec<_> = self
                .time
                .iter()
                .map(|t| t - duration + stats_duration >= 0f64)
                .collect();
            let data: Vec<_> = self
                .forces_and_moments
                .iter()
                .map(|(key, value)| {
                    let force_magnitude: Option<Vec<f64>> = value
                        .iter()
                        .zip(time_filter.iter())
                        .filter(|(_, t)| **t)
                        .map(|(e, _)| e.force.magnitude())
                        .collect();
                    match force_magnitude {
                        Some(ref value) => {
                            let (mean, std) = stats(value);
                            let (min, max) = minmax(value);
                            format!(
                                " {:} & {:.3} & {:.3} & {:.3} & {:.3} \\\\",
                                key.replace("_", " "),
                                mean,
                                std,
                                min,
                                max
                            )
                        }
                        None => format!(" {:} \\\\", key.replace("_", " ")),
                    }
                })
                .collect();
            Some(format!(
                r#"
\begin{{longtable}}{{crrrr}}\toprule
 ELEMENT & MEAN & STD & MIN & MAX \\\hline
{}
\bottomrule
\end{{longtable}}
"#,
                data.join("\n")
            ))
        }
    }
    /// Return a latex table with moment monitors summary
    pub fn moment_latex_table(&self, stats_duration: f64) -> Option<String> {
        let max_value = |x: &[f64]| x.iter().cloned().fold(std::f64::NEG_INFINITY, f64::max);
        let min_value = |x: &[f64]| x.iter().cloned().fold(std::f64::INFINITY, f64::min);
        let minmax = |x: &[f64]| (min_value(x), max_value(x));
        let stats = |x: &[f64]| {
            let n = x.len() as f64;
            let mean = x.iter().sum::<f64>() / n;
            let std = (x.iter().map(|x| x - mean).fold(0f64, |s, x| s + x * x) / n).sqrt();
            (mean, std)
        };
        if self.forces_and_moments.is_empty() {
            None
        } else {
            let duration = self.time.last().unwrap();
            let time_filter: Vec<_> = self
                .time
                .iter()
                .map(|t| t - duration + stats_duration >= 0f64)
                .collect();
            let data: Vec<_> = self
                .forces_and_moments
                .iter()
                .map(|(key, value)| {
                    let moment_magnitude: Option<Vec<f64>> = value
                        .iter()
                        .zip(time_filter.iter())
                        .filter(|(_, t)| **t)
                        .map(|(e, _)| e.moment.magnitude())
                        .collect();
                    match moment_magnitude {
                        Some(ref value) => {
                            let (mean, std) = stats(value);
                            let (min, max) = minmax(value);
                            format!(
                                " {:} & {:.3} & {:.3} & {:.3} & {:.3} \\\\",
                                key.replace("_", " "),
                                mean,
                                std,
                                min,
                                max
                            )
                        }
                        None => format!(" {:} \\\\", key.replace("_", " ")),
                    }
                })
                .collect();
            Some(format!(
                r#"
\begin{{longtable}}{{crrrr}}\toprule
 ELEMENT & MEAN & STD & MIN & MAX \\\hline
{}
\bottomrule
\end{{longtable}}
"#,
                data.join("\n")
            ))
        }
    }
    /// Print out a monitors summary
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
        pkl::to_writer(&mut file, &windloads, Default::default())?;
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
                        .skip(10 * 20)
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
