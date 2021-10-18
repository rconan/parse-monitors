use colorous;
use csv;
use indicatif::ProgressBar;
use parse_monitors::{MonitorsLoader, Vector};
use plotters::prelude::*;
use serde::Deserialize;
use std::path::Path;
use structopt::StructOpt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize)]
pub struct Cover {
    #[allow(dead_code)]
    #[serde(rename = "deltaT (K)")]
    delta_temp: f64,
    #[serde(rename = "X (m)")]
    x: f64,
    #[serde(rename = "Y (m)")]
    y: f64,
    #[serde(rename = "Z (m)")]
    z: f64,
}
pub fn load_cover_xyz<P: AsRef<Path>>(filename: P) -> Result<Vec<Vector>> {
    let mut rdr = csv::Reader::from_path(filename)?;
    let mut cover_xyz: Vec<Vector> = vec![];
    for result in rdr.deserialize() {
        let record: Cover = result?;
        cover_xyz.push(Vector {
            x: Some(record.x),
            y: Some(record.y),
            z: Some(record.z),
        });
    }
    Ok(cover_xyz)
}
pub fn minimize_moment_errors(
    forces: impl Iterator<Item = Vector>,
    moments: impl Iterator<Item = Vector>,
    cover_xyz: &[Vector],
) -> (Vec<usize>, Vec<f64>) {
    forces
        .zip(moments)
        .map(|(f, m)| {
            cover_xyz
                .iter()
                .map(move |r| (&m - &r.cross(&f)).norm_squared().sqrt())
                .enumerate()
                .fold((usize::MAX, f64::INFINITY), |mut c, (k, e)| {
                    if e < c.1 {
                        c.1 = e;
                        c.0 = k;
                    };
                    c
                })
        })
        .unzip()
}

const ANIMATE: bool = true;
const DATA_PATH: &str = "data/m1covers";

#[derive(Debug, StructOpt)]
#[structopt(name = "arms", about = "Estimate force appllication point")]
struct Opt {
    /// Path to the monitor file repository
    path: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    println!("Monitors");
    let monitors = MonitorsLoader::default()
        .data_path(opt.path.clone())
        .header_filter(r"M1cov[1-6]|M1covin[1-6]".to_string())
        .start_time(300f64)
        .load()?;
    monitors.summary();

    let data_path = Path::new(DATA_PATH);

    println!("M1 outer covers");
    println!(" - mean XYZ:");
    let (data, outer_xyz_means): (Vec<_>, Vec<_>) = (1..=6)
        .map(|i| {
            let cover = format!("M1cov{}", i);
            let outer_cover =
                load_cover_xyz(data_path.join(&format!("{}.csv", cover.to_lowercase()))).unwrap();
            let (outer_xyz_idx, _min_moment_errors): (Vec<_>, Vec<_>) = minimize_moment_errors(
                monitors.forces_and_moments[&cover]
                    .iter()
                    .map(|ex| ex.force.clone()),
                monitors.forces_and_moments[&cover]
                    .iter()
                    .map(|ex| ex.moment.clone()),
                &outer_cover,
            );
            //println!("{:#?}", (min_moment_errors, &outer_xyz_idx));
            let outer_xyz_mean = outer_xyz_idx
                .iter()
                .fold(Vector::zero(), |s, &i| &s + &outer_cover[i])
                / outer_xyz_idx.len() as f64;
            println!("{:#}", outer_xyz_mean);
            ((outer_cover, outer_xyz_idx), outer_xyz_mean)
        })
        .unzip();
    let (outer_covers, outer_xyz_ids): (Vec<_>, Vec<_>) = data.into_iter().unzip();

    println!("M1 center covers");
    println!(" - mean XYZ:");
    let (data, center_xyz_means): (Vec<_>, Vec<_>) = (1..=6)
        .map(|i| {
            let cover = format!("M1covin{}", i);
            let center_cover =
                load_cover_xyz(data_path.join(&format!("{}.csv", cover.to_lowercase()))).unwrap();
            let (center_xyz_idx, _min_moment_errors): (Vec<_>, Vec<_>) = minimize_moment_errors(
                monitors.forces_and_moments[&cover]
                    .iter()
                    .map(|ex| ex.force.clone()),
                monitors.forces_and_moments[&cover]
                    .iter()
                    .map(|ex| ex.moment.clone()),
                &center_cover,
            );
            //println!("{:#?}", (min_moment_errors, &center_xyz_idx));
            let center_xyz_mean = center_xyz_idx
                .iter()
                .fold(Vector::zero(), |s, &i| &s + &center_cover[i])
                / center_xyz_idx.len() as f64;
            println!("{:#}", center_xyz_mean);
            ((center_cover, center_xyz_idx), center_xyz_mean)
        })
        .unzip();
    let (center_covers, center_xyz_ids): (Vec<_>, Vec<_>) = data.into_iter().unzip();

    if ANIMATE {
        println!("Animation");
        let pb = ProgressBar::new(200);
        //    let area = SVGBackend::new("m1-covers_3d.svg", (1024, 760)).into_drawing_area();
        let area = BitMapBackend::gif(format!("{}/m1-covers_3d.gif", opt.path), (1024, 1024), 100)
            .unwrap()
            .into_drawing_area();

        for i in usize::max(monitors.len() - 200, 0)..monitors.len() {
            pb.inc(1);
            area.fill(&BLACK)?;
            let x_axis = (0.0..20.0).step(0.5);
            let z_axis = (3.0..8.0).step(0.5);

            let mut chart = ChartBuilder::on(&area)
                .caption(
                    format!("M1 Mirror Cover #1 (T={:7.3}s)", i as f64 * 0.05),
                    ("sans", 20),
                )
                .build_cartesian_3d(x_axis.clone(), z_axis.clone(), x_axis.clone())?;

            chart.with_projection(|mut pb| {
                pb.yaw = 0.8;
                pb.pitch = 0.3;
                pb.scale = 0.8;
                pb.into_matrix()
            });

            chart.configure_axes().draw()?;

            let mut colors = colorous::TABLEAU10.iter().cycle();
            for ((outer_cover, outer_xyz_idx), outer_xyz_mean) in
                (outer_covers.iter().zip(outer_xyz_ids.iter())).zip(outer_xyz_means.iter())
            {
                let color = colors.next().unwrap().as_tuple();
                let rgb = RGBColor(color.0, color.1, color.2);
                chart.draw_series(outer_cover.iter().cloned().map(|xyz| {
                    let v = xyz.into_tuple();
                    Circle::new((v.0, v.2, v.1), 1, rgb.mix(0.5))
                }))?;
                // chart.draw_series(outer_xyz_idx.into_iter().take(1).map(|idx| {
                //     let v = outer_cover[idx].as_tuple();
                //     Circle::new((*v.0, *v.2, *v.1), 4, RED.filled())
                // }))?;
                chart.draw_series(std::iter::once(outer_xyz_idx[i]).map(|idx| {
                    let v = outer_cover[idx].as_tuple();
                    Circle::new((*v.0, *v.2, *v.1), 4, rgb.filled())
                }))?;
                chart.draw_series(std::iter::once(outer_xyz_mean).map(|v| {
                    let (x, y, z) = v.as_tuple();
                    TriangleMarker::new((*x, *z, *y), 6, rgb.filled())
                }))?;
            }

            let mut colors = colorous::TABLEAU10.iter().cycle();
            colors.next();
            for ((center_cover, center_xyz_idx), center_xyz_mean) in
                (center_covers.iter().zip(center_xyz_ids.iter())).zip(center_xyz_means.iter())
            {
                let color = colors.next().unwrap().as_tuple();
                let rgb = RGBColor(color.0, color.1, color.2);
                chart.draw_series(center_cover.iter().cloned().map(|xyz| {
                    let v = xyz.into_tuple();
                    Circle::new((v.0, v.2, v.1), 1, rgb.mix(0.5))
                }))?;
                /*chart.draw_series(center_xyz_idx.into_iter().take(1).map(|idx| {
                    let v = center_cover[idx].as_tuple();
                    Circle::new((*v.0, *v.2, *v.1), 4, RED.filled())
                }))?;*/
                chart.draw_series(std::iter::once(center_xyz_idx[i]).map(|idx| {
                    let v = center_cover[idx].as_tuple();
                    Circle::new((*v.0, *v.2, *v.1), 4, rgb.filled())
                }))?;
                chart.draw_series(std::iter::once(center_xyz_mean).map(|v| {
                    let (x, y, z) = v.as_tuple();
                    TriangleMarker::new((*x, *z, *y), 6, rgb.filled())
                }))?;
            }
            area.present()?;
        }
        pb.finish();
    }
    Ok(())
}
