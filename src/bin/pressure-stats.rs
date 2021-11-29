use parse_monitors::cfd;
use plotters::prelude::*;
use polars::prelude::*;
use rayon::prelude::*;

fn main() -> Result<()> {
    let stats = String::from("Mean");
    let _ = cfd::Baseline::<2021>::default()
        .extras()
        .into_iter()
        .collect::<Vec<cfd::CfdCase<2021>>>()
        .into_par_iter()
        .map(|data_path| {
            let mut df = {
                let filename = "m1_pressure-stats.csv";
                let path = cfd::Baseline::<2021>::path()
                    .join(data_path.to_string())
                    .join(filename);
                CsvReader::from_path(path)?
                    .infer_schema(None)
                    .has_header(true)
                    .finish()?
            };
            df.sort_in_place("Time [s]", false)?;

            let filename = format!("m1_pressure-stats_{}.png", stats.to_lowercase());
            let path = cfd::Baseline::<2021>::path()
                .join(data_path.to_string())
                .join(filename);

            let plot = BitMapBackend::new(&path, (768, 512)).into_drawing_area();
            plot.fill(&WHITE).unwrap();

            let time: Vec<f64> = df.column("Time [s]")?.f64()?.into_no_null_iter().collect();
            let xrange = (*time.first().unwrap(), *time.last().unwrap());

            let cols: Vec<_> = (1..=7)
                .map(|sid| format!("S{} {} [Pa]", sid, stats))
                .collect();
            let df_sub = df.select(cols.iter().map(|s| s.as_str()).collect::<Vec<&str>>())?;

            let min_value: f64 = df_sub.hmin()?.unwrap().min().unwrap();
            let max_value: f64 = df_sub.hmax()?.unwrap().max().unwrap();

            let minmax_padding = 0.;
            let mut chart = ChartBuilder::on(&plot)
                .set_label_area_size(LabelAreaPosition::Left, 60)
                .set_label_area_size(LabelAreaPosition::Bottom, 40)
                .margin(10)
                .build_cartesian_2d(
                    xrange.0..xrange.1,
                    min_value * (1. - minmax_padding)..max_value * (1. + minmax_padding),
                )
                .unwrap();
            chart
                .configure_mesh()
                .x_desc("Time [s]")
                .y_desc(format!("Pressure {}. [Pa]", stats))
                .draw()
                .unwrap();

            let mut colors = colorous::TABLEAU10.iter().cycle();

            for (k, col) in cols.iter().enumerate() {
                let values: Vec<f64> = df_sub.column(&col)?.f64()?.into_no_null_iter().collect();
                let key = format!("S{}", k + 1);
                let color = colors.next().unwrap();
                let rgb = RGBColor(color.r, color.g, color.b);
                chart
                    .draw_series(LineSeries::new(
                        time.iter().zip(values.iter()).map(|(&x, &y)| (x, y)),
                        &rgb,
                    ))
                    .unwrap()
                    .label(key)
                    .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &rgb));
            }
            chart
                .configure_series_labels()
                .border_style(&BLACK)
                .background_style(&WHITE.mix(0.8))
                .position(SeriesLabelPosition::UpperRight)
                .draw()
                .unwrap();
            Ok(())
        })
        .collect::<Result<Vec<()>>>();
    Ok(())
}
