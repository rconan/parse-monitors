use parse_monitors::MonitorsLoader;
use polars::prelude::*;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Setting the path to the directory with the data file: "monitors.csv.z"
    let path_to_data = Path::new("data");
    // Loading the "monitors" file
    let monitors = MonitorsLoader::<2021>::default()
        .data_path(path_to_data)
        .load()?;

    // The HTC are available in the "heat_transfer_coefficients" property
    println!(
        "HTC # of elements: {}",
        monitors.heat_transfer_coefficients.len()
    );

    // For statiscal analysis, it may be a better option to import the data in a polars (https://pola-rs.github.io/polars-book/user-guide/index.html) dataframe
    let htc: DataFrame = monitors
        .heat_transfer_coefficients
        .into_iter()
        .map(|(key, value)| Series::new(&key, value))
        .collect();
    println!("{}", htc.head(None));

    Ok(())
}
