use std::f64::consts;

use colorous;
use parse_monitors::{cfd, cfd::BaselineTrait, Band, DomeSeeing};
use plotters::prelude::*;
use rayon::prelude::*;

// MAIN
fn main() {
    let zenith_angle = cfd::ZenithAngle::Thirty;
    let cfd_cases_21 = cfd::Baseline::<2021>::at_zenith(zenith_angle.clone())
        .into_iter()
        .collect::<Vec<cfd::CfdCase<2021>>>();
    let results: Vec<_> = cfd_cases_21
        .into_par_iter()
        .map(|cfd_case_21| {
            let path_to_case = cfd::Baseline::<2021>::path().join(format!("{}", cfd_case_21));
            let ds_21 = DomeSeeing::load(path_to_case.clone()).unwrap();
            if let (Some(v_pssn), Some(h_pssn)) = (ds_21.pssn(Band::V), ds_21.pssn(Band::H)) {
                let wfe_rms =
                    1e9 * (ds_21.wfe_rms().map(|x| x * x).sum::<f64>() / ds_21.len() as f64).sqrt();
                Some(((cfd_case_21.clone(), wfe_rms, v_pssn, h_pssn),))
            } else {
                None
            }
        })
        .collect();

    let mut data = vec![];
    let mut labels = vec![];
    for (wind, enclosure) in cfd::Baseline::<2021>::configuration(zenith_angle).into_iter() {
        let wind_data: Vec<_> = results
            .iter()
            .cloned()
            .flatten()
            .filter_map(|((case, w, v_pssn, _),)| {
                if case.wind_speed == wind && case.enclosure == enclosure {
                    let (s, c) = case.azimuth.sin_cos();
                    //Some((c * w, s * w))
                    Some((c * (1. - v_pssn), s * (1. - v_pssn)))
                } else {
                    None
                }
            })
            .collect();
        data.push(wind_data);
        labels.push(format!("{:?} {:?} m/s", enclosure, wind));
    }

    let filename = "v-pssn_quipu.png";
    let fig = BitMapBackend::new(filename, (1000, 500)).into_drawing_area();
    fig.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&fig)
        .set_label_area_size(LabelAreaPosition::Left, 50)
        .set_label_area_size(LabelAreaPosition::Bottom, 50)
        .margin(10)
        .build_cartesian_2d(-0.2f64..0.2f64, -0.01f64..0.2f64)
        .unwrap();
    chart.configure_mesh().x_desc("1 - V PSSn").draw().unwrap();

    // SPOKES
    let max_radius = 0.225f64;
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
    let dr = 0.05_f64;
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
    let mut colors = colorous::TABLEAU10.iter().cycle();
    for (k, (this_data, label)) in data.into_iter().zip(labels.into_iter()).enumerate() {
        let this_color = colors.next().unwrap().as_tuple();
        let rgb = RGBColor(this_color.0, this_color.1, this_color.2);
        if k < 2 {
            chart
                .draw_series(
                    this_data
                        .iter()
                        .cloned()
                        .map(|(x, y)| Circle::new((x, y), 8, rgb)),
                )
                .unwrap()
                .label(label)
                .legend(move |(x, y)| Circle::new((x, y), 5, rgb));
        } else {
            chart
                .draw_series(this_data.iter().cloned().map(|(x, y)| {
                    Rectangle::new([(x - 0.0025, y - 0.0025), (x + 0.0025, y + 0.0025)], rgb)
                }))
                .unwrap()
                .label(label)
                .legend(move |(x, y)| Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], rgb));
        }
    }
    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE.mix(0.8))
        .draw()
        .unwrap();

    /*
        let filename = "wfe_rms_wind-rose.png";
        let fig = BitMapBackend::new(filename, (1000, 500)).into_drawing_area();
        fig.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&fig)
            .set_label_area_size(LabelAreaPosition::Left, 50)
            .set_label_area_size(LabelAreaPosition::Bottom, 50)
            .margin(10)
            .build_cartesian_2d(-2000f64..2000f64, -100f64..2000f64)
            .unwrap();
        chart
            .configure_mesh()
            .x_desc("WFE RMS [nm]")
            .draw()
            .unwrap();

        let max_radius = 1800f64;
        for k in 0..5 {
            let (s, c) = (k as f64 * consts::FRAC_PI_4).sin_cos();
            chart
                .draw_series(LineSeries::new(
                    (0..2).map(|x| (x as f64 * max_radius * c, x as f64 * max_radius * s)),
                    &BLACK,
                ))
                .unwrap();
        }
        let dd = 20f64;
        let dr = 500f64;
        for k in 0..4 {
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

        let mut colors = colorous::TABLEAU10.iter().cycle();
        for (k, (this_data, label)) in data.into_iter().zip(labels.into_iter()).enumerate() {
            let this_color = colors.next().unwrap().as_tuple();
            let rgb = RGBColor(this_color.0, this_color.1, this_color.2);
            if k < 2 {
                chart
                    .draw_series(
                        this_data
                            .iter()
                            .cloned()
                            .map(|(x, y)| Circle::new((x, y), 8, rgb.filled())),
                    )
                    .unwrap()
                    .label(label)
                    .legend(move |(x, y)| Circle::new((x, y), 5, rgb.filled()));
            } else {
                chart
                    .draw_series(this_data.iter().cloned().map(|(x, y)| {
                        Rectangle::new([(x - 30., y - 30.), (x + 30., y + 30.)], rgb.filled())
                    }))
                    .unwrap()
                    .label(label)
                    .legend(move |(x, y)| {
                        Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], rgb.filled())
                    });
            }
        }
        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.8))
            .draw()
            .unwrap();
    */
}
