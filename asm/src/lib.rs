use polars::prelude::*;

pub mod pressure;
pub mod refraction_index;

#[derive(Debug, Default)]
pub struct Stats {
    sample_name: String,
    sample_size: usize,
    mean: Option<f64>,
    median: Option<f64>,
    var: Option<f64>,
    max: Option<f64>,
    min: Option<f64>,
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

pub trait StatsDataFrame {
    fn new(stats_data: Vec<Stats>) -> Result<DataFrame> {
        let series = vec![Series::new(
            "case",
            &stats_data
                .iter()
                .map(|x| x.sample_name.to_owned())
                .collect::<Vec<String>>(),
        )];
        DataFrame::new(series)
    }
}
