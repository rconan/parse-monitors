use indicatif::ParallelProgressIterator;
use parse_monitors::{
    cfd::{self, BaselineTrait},
    pressure::{rtree::Node, Telescope},
};
use polars::prelude::*;
use rayon::prelude::*;
use serde::Serialize;
use std::fs::File;

#[derive(Serialize)]
pub struct Data {
    telescope_mean_pressure: f64,
    gir_mean_pressure: f64,
    gir_panels: Vec<Vec<Node>>,
}

#[derive(Default, Debug, Serialize)]
pub struct GMACSWindLoads {
    pub gir_integrated_force: [f64; 3],
    mean: Vec<f64>,
    median: Vec<f64>,
    std: Vec<f64>,
    minmax: Vec<(f64, f64)>,
    integrated_force: Vec<[f64; 3]>,
    center_of_pressure: Vec<[f64; 3]>,
}

impl GMACSWindLoads {
    pub fn new(gir_integrated_force: [f64; 3]) -> Self {
        Self {
            gir_integrated_force,
            ..Default::default()
        }
    }
    pub fn push(&mut self, panel: Telescope, mean_pressure: f64) {
        let forces: Vec<[f64; 3]> = panel
            .pressure_iter()
            .map(|p| *p - mean_pressure)
            .zip(panel.area_ijk_iter())
            .map(|(p, a)| [p * a[0], p * a[1], p * a[2]])
            .collect();
        let mut forces_mag: Vec<f64> = forces
            .iter()
            .map(|f| f.iter().map(|&x| x * x).sum::<f64>().sqrt())
            .collect();
        forces_mag.sort_by(|&a, b| a.partial_cmp(b).unwrap());
        let n = forces_mag.len();
        let mean_forces_mag = forces_mag.iter().sum::<f64>() / n as f64;
        let std_forces_mag = (forces_mag
            .iter()
            .map(|x| x - mean_forces_mag)
            .map(|x| x * x)
            .sum::<f64>()
            / n as f64)
            .sqrt();
        let median_forces_mag = if n % 2 == 0 {
            0.5 * (forces_mag[n / 2 - 1] + forces_mag[n / 2])
        } else {
            forces_mag[(n - 1) / 2]
        };
        self.mean.push(mean_forces_mag);
        self.median.push(median_forces_mag);
        self.std.push(std_forces_mag);
        self.minmax.push((forces_mag[0], forces_mag[n - 1]));
        let (integrated_force, mut center_of_pressure) = forces.iter().zip(panel.xyz_iter()).fold(
            ([0f64; 3], [0f64; 3]),
            |(mut fi, mut cfi), (f, xyz)| {
                for k in 0..3 {
                    fi[k] += f[k];
                    cfi[k] += f[k] * xyz[k];
                }
                (fi, cfi)
            },
        );
        center_of_pressure
            .iter_mut()
            .zip(&integrated_force)
            .for_each(|(cp, fi)| *cp /= fi);
        self.integrated_force.push(integrated_force);
        self.center_of_pressure.push(center_of_pressure);
    }
}

fn main() -> anyhow::Result<()> {
    let cfd_case = cfd::Baseline::<2021>::default()
        .into_iter()
        .nth(37)
        .unwrap();
    let data_file = cfd::CfdDataFile::<2021>::TelescopePressure;
    let telescope_pressure_files = data_file.glob(cfd_case)?;
    println!(
        "CFD CASE: {} ({} pressure files)",
        cfd_case,
        telescope_pressure_files.len()
    );

    let data = telescope_pressure_files
        .chunks(100)
        .flat_map(|files| {
            files
                .into_par_iter()
                .progress()
                .map(|file| {
                    let telescope = Telescope::from_path(file).unwrap();
                    // println!("{telescope}");
                    let telescope_mean_pressure = telescope.mean_pressure();
                    let rtree = telescope.to_rtree();

                    // let mut gir_panels = vec![]; // index: 0=>a , 1=>b, 2=>c
                    let data_path = std::path::Path::new("data");

                    let df = CsvReader::from_path(data_path.join("GIRVol_aera.csv"))
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
                            let node = if let ((Some(x), Some(y)), Some(z)) = xyz {
                                rtree.locate_at_point(&[x, y, z]).map(|node| node.clone())
                            } else {
                                None
                            };
                            // if node.is_none() {
                            // panic!("node not found")
                            // }
                            node
                        })
                        .collect::<Vec<Node>>()
                        .into();
                    let gir_mean_pressure: f64 =
                        gir.pressure_iter().map(|p| p).sum::<f64>() / n_sample as f64;
                    let gir_integrated_force = gir.pressure_iter().zip(gir.area_ijk_iter()).fold(
                        [0f64; 3],
                        |mut fi, (p, a)| {
                            for k in 0..3 {
                                fi[k] += p * a[k];
                            }
                            fi
                        },
                    );
                    let mut gmacs = GMACSWindLoads::new(gir_integrated_force);

                    for gir_csv in ["gir_a.csv", "gir_b.csv", "gir_c.csv"] {
                        let df = CsvReader::from_path(data_path.join(format!(
                            "{}_{}",
                            cfd_case.to_string(),
                            gir_csv
                        )))
                        .unwrap()
                        .has_header(true)
                        .finish()
                        .unwrap();
                        // println!("Dataframe shape: {:?}", df.shape());
                        // println!("{}", df.head(Some(10)));

                        // let now = Instant::now();
                        let gir_panel: Telescope = df[3]
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
                        gmacs.push(gir_panel, gir_mean_pressure);
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

                    /*                     Data {
                        telescope_mean_pressure,
                        gir_mean_pressure,
                        gir_panels,
                    } */
                    gmacs
                })
                .collect::<Vec<GMACSWindLoads>>()
        })
        .collect::<Vec<GMACSWindLoads>>();

    let cfd_path = cfd::Baseline::<2021>::default_path();
    let cfd_case_path = cfd_path.join(cfd_case.to_string());
    serde_pickle::to_writer(
        &mut File::create(cfd_case_path.join("gmacs.pkl")).unwrap(),
        &data,
        Default::default(),
    )?;

    Ok(())
}
