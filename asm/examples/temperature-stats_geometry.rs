use bzip2::read::BzDecoder;
use geotrans::*;
use parse_monitors::{cfd, pressure::Pressure};
use plotters::prelude::*;
use polars::prelude::*;
use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
};

const ASM_R: f64 = 0.52;
const R: f64 = 1.2;
const H: f64 = 0.5;

fn outline(dr: f64, radius: f64) -> impl Iterator<Item = (f64, f64)> {
    let n = (2. * std::f64::consts::PI * radius / dr).round() as usize;
    (0..=n).map(move |i| {
        let o = 2. * std::f64::consts::PI * (i as f64) / (n as f64);
        let (y, x) = o.sin_cos();
        (radius * x, radius * y)
    })
}

fn main() -> anyhow::Result<()> {
    let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
    let path = cfd::CfdDataFile::<2021>::TemperatureField
        .glob(cfd_case)?
        .last()
        .unwrap()?;

    let es_df = {
        let df = {
            let csv_file = File::open(&path)?;
            let mut contents = String::new();
            BzDecoder::new(BufReader::new(csv_file)).read_to_string(&mut contents)?;
            CsvReader::new(Cursor::new(contents.as_bytes()))
                .with_path(Some(path))
                .infer_schema(None)
                .has_header(true)
                .finish()?
        };
        let radii = {
            let x = df.column("X (m)")?;
            let y = df.column("Y (m)")?;
            &(x * x) + &(y * y)
        };
        let mask: ChunkedArray<BooleanType> = radii
            .lt(R * R)
            .into_iter()
            .zip(radii.gt(H * H).into_iter())
            .filter_map(|(l, r)| l.zip(r).map(|(l, r)| l && r))
            .collect();
        df.filter(&mask)?
    };

    let x = Vec::from(es_df.column("X (m)")?.f64()?)
        .into_iter()
        .collect::<Option<Vec<f64>>>()
        .unwrap();
    let y = Vec::from(es_df.column("Y (m)")?.f64()?)
        .into_iter()
        .collect::<Option<Vec<f64>>>()
        .unwrap();
    let z = Vec::from(es_df.column("Z (m)")?.f64()?)
        .into_iter()
        .collect::<Option<Vec<f64>>>()
        .unwrap();

    /*
        type M12 = geotrans::M2;
        let geometry = "M2p.csv.bz2";
        let csv_pressure = Pressure::<M12>::decompress(path.to_path_buf()).unwrap();
        let csv_geometry = Pressure::<M12>::decompress(path.with_file_name(geometry)).unwrap();
        let mut pressures = Pressure::<M12>::load(csv_pressure, csv_geometry).unwrap();
    */
    let drawing_area = BitMapBackend::new("pressure-stats_xy.png", (768, 768)).into_drawing_area();
    drawing_area.fill(&WHITE).unwrap();
    let lim = 1.6_f64;
    let mut chart = ChartBuilder::on(&drawing_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(-lim..lim, -lim..lim)
        .unwrap();
    chart.configure_mesh().draw().unwrap();

    for sid in 1..=7 {
        chart
            .draw_series(LineSeries::new(
                outline(1e-2, ASM_R).map(|(x, y)| {
                    let u = [x, y, 0.].to(Segment::<M2>::new(sid)).unwrap();
                    (u[0], u[1])
                }),
                &BLACK,
            ))
            .unwrap();
    }
    chart
        .draw_series(LineSeries::new(outline(1e-2, R), &RED))
        .unwrap();
    chart
        .draw_series(
            x.into_iter()
                .zip(y.into_iter())
                .map(|point| Circle::new(point, 2, BLACK.filled())),
        )
        .unwrap();

    println!("{}", es_df);
    Ok(())
}
