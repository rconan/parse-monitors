use crate::{Exertion, Monitors, MonitorsLoader};
#[cfg(feature = "plot")]
use plotters::prelude::*;
use std::{
    collections::{BTreeMap, VecDeque},
    fs::File,
    path::Path,
};

/// Mirror data loader
pub struct MirrorLoader<P: AsRef<Path>> {
    mirror: Mirror,
    path: P,
    time_range: (f64, f64),
    net_force: bool,
}
impl<P: AsRef<Path>> MirrorLoader<P> {
    fn new(mirror: Mirror, path: P) -> Self {
        MirrorLoader {
            mirror,
            path,
            time_range: (0f64, f64::INFINITY),
            net_force: false,
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
    pub fn net_force(self) -> Self {
        Self {
            net_force: true,
            ..self
        }
    }
    pub fn load(self) -> Result<Mirror, Box<dyn std::error::Error>> {
        let mut mirror = self.mirror;
        let (filename, time, force) = match &mut mirror {
            Mirror::M1 { time, force } => ("center_of_pressure.csv", time, force),
            Mirror::M2 { time, force } => ("M2_segments_force.csv", time, force),
        };
        let path = Path::new(self.path.as_ref());
        if let Ok(csv_file) = File::open(&path.join(filename)) {
            let mut rdr = csv::Reader::from_reader(csv_file);
            for result in rdr.deserialize() {
                let record: (f64, Vec<([f64; 3], ([f64; 3], [f64; 3]))>) = result?;
                let t = record.0;
                if t < self.time_range.0 - 1. / 40. || t > self.time_range.1 + 1. / 40. {
                    continue;
                };
                let mut record_iter = record.1.into_iter();
                if let Some(t_b) = time.back() {
                    if t >= *t_b {
                        time.push_back(t);
                        for fm in force.values_mut() {
                            fm.push_back(record_iter.next().unwrap().into())
                        }
                    } else {
                        if let Some(index) = time.iter().rposition(|&x| x < t) {
                            time.insert(index + 1, t);
                            for fm in force.values_mut() {
                                fm.insert(index + 1, record_iter.next().unwrap().into())
                            }
                        } else {
                            time.push_front(t);
                            for fm in force.values_mut() {
                                fm.push_front(record_iter.next().unwrap().into())
                            }
                        }
                    }
                } else {
                    time.push_back(t);
                    for fm in force.values_mut() {
                        fm.push_back(record_iter.next().unwrap().into())
                    }
                }
            }
            if self.net_force {
                if let Mirror::M1 { time, force } = &mut mirror {
                    let ts = *time.front().unwrap();
                    let te = *time.back().unwrap();
                    let monitors = MonitorsLoader::<2021>::default()
                        .data_path(path)
                        .header_filter("M1cell".to_string())
                        .start_time(ts)
                        .end_time(te)
                        .load()?;
                    let m1_cell = &monitors.forces_and_moments["M1cell"];
                    assert_eq!(
                        time.len(),
                        m1_cell.len(),
                        "{:?} {:?}/{:?}: M1 segments and M1 cell # of sample do not match",
                        (ts, te),
                        (monitors.time[0], monitors.time.last().unwrap()),
                        path
                    );
                    for v in force.values_mut() {
                        for (e, cell) in v.iter_mut().zip(m1_cell) {
                            let mut f = &mut e.force;
                            f += &(&cell.force / 7f64).unwrap();
                            let mut m = &mut e.moment;
                            m += &(&cell.moment / 7f64).unwrap();
                        }
                    }
                }
            }
            Ok(mirror)
        } else {
            Err(format!("Cannot open {:?}", &path).into())
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
    pub fn m1<P: AsRef<Path>>(path: P) -> MirrorLoader<P> {
        let mut force: BTreeMap<String, VecDeque<Exertion>> = BTreeMap::new();
        (1..=7).for_each(|k| {
            force.entry(format!("S{}", k)).or_default();
        });
        MirrorLoader::new(
            Mirror::M1 {
                time: VecDeque::new(),
                force,
            },
            path,
        )
    }
    pub fn m2<P: AsRef<Path>>(path: P) -> MirrorLoader<P> {
        let mut force: BTreeMap<String, VecDeque<Exertion>> = BTreeMap::new();
        (1..=7).for_each(|k| {
            force.entry(format!("S{}", k)).or_default();
        });
        MirrorLoader::new(
            Mirror::M2 {
                time: VecDeque::new(),
                force,
            },
            path,
        )
    }
    /// Keeps only the last `period` seconds of the monitors
    pub fn keep_last(&mut self, period: usize) -> &mut Self {
        let i = self.len() - period * crate::FORCE_SAMPLING_FREQUENCY as usize;
        let _: Vec<_> = self.time_mut().drain(..i).collect();
        for value in self.exertion_mut() {
            let _: Vec<_> = value.drain(..i).collect();
        }
        self
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
    pub fn len(&self) -> usize {
        self.time().len()
    }
    pub fn time(&self) -> &VecDeque<f64> {
        match self {
            Mirror::M1 { time, .. } => time,
            Mirror::M2 { time, .. } => time,
        }
    }
    pub fn time_mut(&mut self) -> &mut VecDeque<f64> {
        match self {
            Mirror::M1 { time, .. } => time,
            Mirror::M2 { time, .. } => time,
        }
    }
    pub fn forces_and_moments(&self) -> &BTreeMap<String, VecDeque<Exertion>> {
        match self {
            Mirror::M1 { force, .. } => force,
            Mirror::M2 { force, .. } => force,
        }
    }
    pub fn forces_and_moments_mut(&mut self) -> &mut BTreeMap<String, VecDeque<Exertion>> {
        match self {
            Mirror::M1 { force, .. } => force,
            Mirror::M2 { force, .. } => force,
        }
    }
    pub fn exertion(&self) -> impl Iterator<Item = &VecDeque<Exertion>> {
        match self {
            Mirror::M1 { force, .. } => force.values(),
            Mirror::M2 { force, .. } => force.values(),
        }
    }
    pub fn exertion_mut(&mut self) -> impl Iterator<Item = &mut VecDeque<Exertion>> {
        match self {
            Mirror::M1 { force, .. } => force.values_mut(),
            Mirror::M2 { force, .. } => force.values_mut(),
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
        if self.forces_and_moments().is_empty() {
            None
        } else {
            let duration = self.time().back().unwrap();
            let time_filter: Vec<_> = self
                .time()
                .iter()
                .map(|t| t - duration + stats_duration - crate::FORCE_SAMPLING > 0f64)
                .collect();
            let data: Vec<_> = self
                .forces_and_moments()
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
            Some(data.join("\n"))
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
        if self.forces_and_moments().is_empty() {
            None
        } else {
            let duration = self.time().back().unwrap();
            let time_filter: Vec<_> = self
                .time()
                .iter()
                .map(|t| t - duration + stats_duration - crate::FORCE_SAMPLING > 0f64)
                .collect();
            let data: Vec<_> = self
                .forces_and_moments()
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
            Some(data.join("\n"))
        }
    }
    #[cfg(feature = "plot")]
    pub fn plot_forces(&self, filename: Option<&str>) {
        if self.forces_and_moments().is_empty() {
            println!("Empty mirror");
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

        let plot =
            BitMapBackend::new(filename.unwrap_or("FORCE.png"), (768, 512)).into_drawing_area();
        plot.fill(&WHITE).unwrap();

        let (min_values, max_values): (Vec<_>, Vec<_>) = self
            .forces_and_moments()
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
        let xrange = (*self.time().front().unwrap(), *self.time().back().unwrap());
        let minmax_padding = 0.1;
        let mut chart = ChartBuilder::on(&plot)
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .margin(10)
            .build_cartesian_2d(
                xrange.0..xrange.1 * (1. + 1e-2),
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

        for (key, values) in self.forces_and_moments().iter() {
            let color = colors.next().unwrap();
            let rgb = RGBColor(color.r, color.g, color.b);
            chart
                .draw_series(LineSeries::new(
                    self.time()
                        .iter()
                        .zip(values.iter())
                        //.skip(10 * 20)
                        .map(|(&x, y)| (x, y.force.magnitude().unwrap())),
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
