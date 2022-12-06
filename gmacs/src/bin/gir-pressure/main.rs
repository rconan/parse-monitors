use parse_monitors::{
    cfd::{self},
    pressure::{rtree::Node, Telescope},
};
use polars::prelude::*;

fn main() -> anyhow::Result<()> {
    let cfd_case = cfd::Baseline::<2021>::default()
        .into_iter()
        .nth(25)
        .unwrap();
    let data_file = cfd::CfdDataFile::<2021>::TelescopePressure;
    for file in data_file.glob(cfd_case)?.iter().last() {
        let telescope = Telescope::from_path(&file).unwrap();
        // println!("{telescope}");
        // let telescope_mean_pressure = telescope.mean_pressure();
        let rtree = telescope.to_rtree();

        // let mut gir_panels = vec![]; // index: 0=>a , 1=>b, 2=>c

        let df = CsvReader::from_path("GIRVol_aera.csv")
            .unwrap()
            .has_header(true)
            .finish()
            .unwrap();
        let (n_sample, _) = df.shape();
        let gir: Telescope = df[3]
            .f64()
            .unwrap()
            .into_iter()
            .zip(df[4].f64().unwrap())
            .zip(df[5].f64().unwrap())
            .filter_map(|xyz| {
                if let ((Some(x), Some(y)), Some(z)) = xyz {
                    rtree.locate_at_point(&[x, y, z]).map(|node| node.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<Node>>()
            .into();
        let gir_mean_pressure: f64 = gir.pressure_iter().map(|p| p).sum::<f64>() / n_sample as f64;
        println!("GIR MEAN PRESSURE ({file:?}): {gir_mean_pressure}Pa");
    }

    Ok(())
}
