use std::fs::File;

use parse_monitors::{
    cfd::{self, BaselineTrait},
    pressure::{rtree::Node, Telescope},
};
use polars::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct Data {
    mean_pressure: f64,
    gir_panels: Vec<Vec<Node>>,
}

fn main() -> anyhow::Result<()> {
    let cfd_case = cfd::Baseline::<2021>::default().into_iter().nth(0).unwrap();
    let data_file = cfd::CfdDataFile::<2021>::TelescopePressure;
    let telescope_pressure_files = data_file.glob(cfd_case)?;

    dbg!(&telescope_pressure_files);

    let data = telescope_pressure_files
        .into_iter()
        .take(1)
        .map(|file| {
            let telescope = Telescope::from_path(file).unwrap();
            println!("{telescope}");
            let mean_pressure = telescope.mean_pressure();
            let rtree = telescope.to_rtree();

            let mut gir_panels = vec![]; // index: 0=>a , 1=>b, 2=>c

            for gir_csv in ["gir_a.csv", "gir_b.csv", "gir_c.csv"] {
                let df = CsvReader::from_path(gir_csv)
                    .unwrap()
                    .has_header(true)
                    .finish()
                    .unwrap();
                // println!("Dataframe shape: {:?}", df.shape());
                // println!("{}", df.head(Some(10)));

                // let now = Instant::now();
                gir_panels.push(
                    df[3]
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
                        .collect::<Vec<_>>(),
                );
                // let (n_sample, _) = df.shape();
                // let n_node = gir_panels.last().unwrap().len();
                /*                 println!(
                    "Identified {} nodes out of {} in {}ms",
                    n_node,
                    n_sample,
                    now.elapsed().as_millis()
                ); */

                // dbg!(nodes[0]);
            }
            Data {
                mean_pressure,
                gir_panels,
            }
        })
        .collect::<Vec<Data>>();

    let cfd_path = cfd::Baseline::<2021>::default_path();
    let cfd_case_path = cfd_path.join(cfd_case.to_string());
    dbg!(&cfd_case_path);
    serde_pickle::to_writer(
        &mut File::create(cfd_case_path.join("gir_panels.pkl")).unwrap(),
        &data,
        Default::default(),
    )?;

    Ok(())
}
