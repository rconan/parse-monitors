use std::{fs::File, time::Instant};

use parse_monitors::pressure::{rtree::Node, Telescope};
use polars::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct Data<'a> {
    mean_pressure: f64,
    gir_panels: Vec<Vec<&'a Node>>,
}

fn main() -> anyhow::Result<()> {
    let telescope =
        Telescope::from_path("../../data/Telescope_p_telescope_7.000000e+02.csv.z").unwrap();
    println!("{telescope}");
    let mean_pressure = telescope.mean_pressure();
    let rtree = telescope.to_rtree();

    let mut gir_panels = vec![]; // index: 0=>a , 1=>b, 2=>c

    for gir_csv in ["gir_a.csv", "gir_b.csv", "gir_c.csv"] {
        let df = CsvReader::from_path(gir_csv)?.has_header(true).finish()?;
        println!("Dataframe shape: {:?}", df.shape());
        // println!("{}", df.head(Some(10)));

        let now = Instant::now();
        gir_panels.push(
            df[3]
                .f64()?
                .into_iter()
                .zip(df[4].f64()?)
                .zip(df[5].f64()?)
                .filter_map(|xyz| {
                    if let ((Some(x), Some(y)), Some(z)) = xyz {
                        rtree.locate_at_point(&[x, y, z])
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        );
        let (n_sample, _) = df.shape();
        let n_node = gir_panels.last().unwrap().len();
        println!(
            "Identified {} nodes out of {} in {}ms",
            n_node,
            n_sample,
            now.elapsed().as_millis()
        );

        // dbg!(nodes[0]);
    }
    let data = Data {
        mean_pressure,
        gir_panels,
    };
    serde_pickle::to_writer(
        &mut File::create("gir_panels.pkl")?,
        &data,
        Default::default(),
    )?;

    Ok(())
}
