use std::{fs::File, time::Instant};

use parse_monitors::pressure::Telescope;
use polars::prelude::*;

fn main() -> anyhow::Result<()> {
    let telescope =
        Telescope::from_path("../../data/Telescope_p_telescope_7.000000e+02.csv.z").unwrap();
    println!("{telescope}");
    let rtree = telescope.to_rtree();

    let df = CsvReader::from_path("GIRVol_aera.csv")?
        .has_header(true)
        .finish()?;
    println!("Dataframe shape: {:?}", df.shape());
    println!("{}", df.head(Some(10)));

    let now = Instant::now();
    let nodes: Vec<_> = df[3]
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
        .collect();
    let (n_sample, _) = df.shape();
    let n_node = nodes.len();
    println!(
        "Identified {} nodes out of {} in {}ms",
        n_node,
        n_sample,
        now.elapsed().as_millis()
    );

    dbg!(nodes[0]);

    serde_pickle::to_writer(
        &mut File::create("gir_nodes.pkl")?,
        &nodes,
        Default::default(),
    )?;

    Ok(())
}
