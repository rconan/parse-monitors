use std::iter::FromIterator;

use polars::prelude::*;

pub mod pressure;
pub mod refraction_index;

type StatsData = Option<f64>;

#[derive(Debug, Default)]
pub struct Stats {
    sample_name: String,
    sample_size: usize,
    mean: StatsData,
    median: StatsData,
    var: StatsData,
    max: StatsData,
    min: StatsData,
}

impl From<ChunkedArray<Float64Type>> for Stats {
    fn from(data: ChunkedArray<Float64Type>) -> Self {
        Self {
            sample_name: data.name().to_string(),
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
    Vec<String>,
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
                    s.sample_name,
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
