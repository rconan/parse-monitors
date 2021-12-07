use std::{
    iter::FromIterator,
    path::{Path, PathBuf},
};

use polars::prelude::*;

pub mod pressure;
pub mod refraction_index;

type StatsData = Option<f64>;

#[derive(Debug, Default)]
pub struct Stats {
    time_stamp: StatsData,
    sample_size: usize,
    mean: StatsData,
    median: StatsData,
    var: StatsData,
    max: StatsData,
    min: StatsData,
}

impl From<(StatsData, ChunkedArray<Float64Type>)> for Stats {
    fn from(
        (time_stamp, data): (
            StatsData,
            polars::prelude::ChunkedArray<polars::prelude::Float64Type>,
        ),
    ) -> Self {
        Self {
            time_stamp,
            sample_size: data.len(),
            mean: data.mean(),
            median: data.median(),
            var: data.var(),
            max: data.max(),
            min: data.min(),
        }
    }
}

type StatsTuple = (
    Vec<StatsData>,
    (
        Vec<u32>,
        (
            Vec<StatsData>,
            (
                Vec<StatsData>,
                (Vec<StatsData>, (Vec<StatsData>, Vec<StatsData>)),
            ),
        ),
    ),
);
impl FromIterator<Stats> for Result<DataFrame> {
    fn from_iter<T: IntoIterator<Item = Stats>>(iter: T) -> Self {
        let (s1, (s2, (s3, (s4, (s5, (s6, s7)))))): StatsTuple = iter
            .into_iter()
            .map(|s| {
                (
                    s.time_stamp,
                    (
                        s.sample_size as u32,
                        (s.mean, (s.median, (s.var, (s.max, s.min)))),
                    ),
                )
            })
            .unzip();
        let mut series = vec![Series::new("name", &s1), Series::new("size", &s2)];
        series.extend(
            ["median", "mean", "var", "min", "max"]
                .into_iter()
                .zip([s4, s3, s5, s7, s6].into_iter())
                .map(|(name, data)| Series::new(name, &data))
                .collect::<Vec<Series>>(),
        );
        DataFrame::new(series)
    }
}

pub fn file_timestamp(path: PathBuf, pattern: &str) -> Option<f64> {
    Path::new(path.file_stem()?)
        .file_stem()?
        .to_str()?
        .replace(pattern, "")
        .parse::<f64>()
        .ok()
        .map(|x| (x * 1e3).round() * 1e-3)
}
