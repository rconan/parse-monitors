use colorous;
use parse_monitors::cfd;
use plotters::prelude::*;
use polars::prelude::*;
use std::f64::consts;
use strum::IntoEnumIterator;

fn main() -> anyhow::Result<()> {
    let df = CsvReader::from_path("pressure_std.csv")?
        .infer_schema(None)
        .has_header(true)
        .finish()?;
    println!("{}", df);
    let mask: Vec<bool> = Vec::from(df.column("case")?.utf8()?)
        .into_iter()
        .filter_map(|s| s.map(|s| s.starts_with("zen00") && !s.ends_with("CD7")))
        .collect();
    let zen_df = df.filter(&ChunkedArray::new_from_slice("", &mask))?;
    println!("{}", zen_df);

    let cases = Vec::from(zen_df.column("case")?.utf8()?)
        .into_iter()
        .collect::<Option<Vec<&str>>>()
        .unwrap();
    let pressure = Vec::from(zen_df.column("pressure std [Pa]")?.f64()?)
        .into_iter()
        .collect::<Option<Vec<f64>>>()
        .unwrap();
    let results = cases.iter().zip(pressure);

    let zenith_angle = cfd::ZenithAngle::Zero;
    //    let mut data = vec![];
    //    let mut labels = vec![];
    for (wind, enclosure) in cfd::Baseline::<2021>::configuration(zenith_angle).into_iter() {
        cfd::ZenithAngle::iter();
        //        for azimuth in cfd::Azimuth::iter() {}
    }
    /*

        let max_std: Option<f64> = zen_df.column("pressure std [Pa]")?.max();
        let lim = max_std.unwrap().ceil();

        let filename = "zen00_pressure-std_quipu.png";
        let fig = BitMapBackend::new(filename, (1000, 500)).into_drawing_area();
        fig.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&fig)
            .set_label_area_size(LabelAreaPosition::Left, 50)
            .set_label_area_size(LabelAreaPosition::Bottom, 50)
            .margin(10)
            .build_cartesian_2d(-lim..lim, -0.01f64..lim)
            .unwrap();
        chart
            .configure_mesh()
            .x_desc("Pressure Std. [Pa]")
            .draw()
            .unwrap();

        // SPOKES
        let max_radius = lim;
        for k in 0..5 {
            let (s, c) = (k as f64 * consts::FRAC_PI_4).sin_cos();
            chart
                .draw_series(LineSeries::new(
                    (0..2).map(|x| (x as f64 * max_radius * c, x as f64 * max_radius * s)),
                    &BLACK,
                ))
                .unwrap();
        }
        // ARCS
        let dd = 0.01_f64;
        let dr = 0.2_f64;
        for k in 1..5 {
            let radius = k as f64 * dr;
            let n = (consts::PI * radius / dd).round() as usize;
            chart
                .draw_series(LineSeries::new(
                    (0..n).map(|k| {
                        let (s, c) = (k as f64 * consts::PI / (n - 1) as f64).sin_cos();
                        (radius * c, radius * s)
                    }),
                    &BLACK,
                ))
                .unwrap();
        }

        let labels = Vec::from(zen_df.column("case")?.utf8()?);
        let data = Vec::from(zen_df.column("pressure std [Pa]")?.f64()?);

        let mut colors = colorous::TABLEAU10.iter().cycle();
        for (k, (this_data, label)) in data.into_iter().zip(labels.into_iter()).enumerate() {
            let this_color = colors.next().unwrap().as_tuple();
            let rgb = RGBColor(this_color.0, this_color.1, this_color.2);
            if k < 2 {
                chart
                    .draw_series(
                        this_data
                            .as_ref()
                            .unwrap()
                            .iter()
                            .cloned()
                            .map(|(x, y)| Circle::new((x, y), 8, rgb)),
                    )
                    .unwrap()
                    .label(label.unwrap())
                    .legend(move |(x, y)| Circle::new((x, y), 5, rgb));
            } else {
                chart
                    .draw_series(this_data.iter().cloned().map(|(x, y)| {
                        Rectangle::new([(x - 0.0025, y - 0.0025), (x + 0.0025, y + 0.0025)], rgb)
                    }))
                    .unwrap()
                    .label(label.unwrap())
                    .legend(move |(x, y)| Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], rgb));
            }
        }
        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.8))
            .draw()
            .unwrap();
    */
    Ok(())
}
