//! DOME SEEING ANALYSYS
//!
//! Make the dome seeing plots unless the environment variable `CFD_PLOTS` is set to `NO`
//! It must be run as root i.e. `sudo -E ./target/release/dome-seeing`

// use indicatif::ParallelProgressIterator;
use parse_monitors::{
    Band, DomeSeeing,
    cfd::{self, BaselineTrait, CfdCase},
    report::ReportError,
};
use rayon::prelude::*;
use std::env;

fn make_figure(data: Vec<Vec<(f64, Vec<f64>)>>, labels: Vec<&str>, filename: &str, ylabel: &str) {
    log::info!("making figure: {filename}");
    let cfd_plots = env::var("CFD_PLOTS").unwrap_or_else(|_| "YES".to_string());
    if cfd_plots != "YES" {
        return;
    }
    let mut config = complot::Config::new()
        .filename(filename)
        .xaxis(complot::Axis::new().label("Time [s]"))
        .yaxis(complot::Axis::new().label(ylabel));
    config.auto_range(
        data.iter()
            .map(|x| x.as_slice())
            .collect::<Vec<&[(f64, Vec<f64>)]>>(),
    );
    let mut data_iter: Vec<Box<(dyn Iterator<Item = (f64, Vec<f64>)> + 'static)>> = vec![];
    for val in data.into_iter() {
        data_iter.push(Box::new(val.into_iter()))
    }
    let kinds: Vec<_> = labels
        .into_iter()
        .map(|l| complot::Kind::Plot(Some(l.into())))
        .collect();
    let _: complot::Combo = From::<complot::Complot>::from((data_iter, kinds, Some(config)));
}
// MAIN
pub fn taks<const R: u32, const Y: u32>(cfd_cases: &[CfdCase<Y>]) -> Result<(), ReportError> {
    let cfd_cases_21 = cfd_cases.to_vec();
    let root = cfd::Baseline::<Y>::path()?;
    let legends = format!("{Y}");
    let other_legends = format!("{R}");
    let wfe_labels: Option<Vec<&str>> = Some(vec![&legends, &other_legends]);
    let pssn_labels: Option<Vec<&str>> = Some(vec![&legends, &other_legends]);
    // let truncate = Some((
    //     Some(cfd::CfdCase::new(
    //         cfd::ZenithAngle::Thirty,
    //         cfd::Azimuth::OneThirtyFive,
    //         cfd::Enclosure::OpenStowed,
    //         cfd::WindSpeed::Seven,
    //     )),
    //     290 * 5,
    // ));
    // let n_cases = cfd_cases_21.len() as u64;
    // dbg!(n_cases);
    let _results: Vec<Option<((String, f64, f64), Option<(String, f64, f64)>)>> = cfd_cases_21
        .into_par_iter()
        //.progress_count(n_cases)
        .map(|cfd_case_current| {
            let path_to_case = root.join(format!("{}", cfd_case_current));
            let ds_21 = DomeSeeing::load(path_to_case.clone()).expect(&format!(
                "Failed to load dome seeing data from {:?}",
                path_to_case.clone()
            ));
            // dbg!(ds_21.len());
            // match &truncate {
            //     Some((Some(cfd_case), len)) => {
            //         if cfd_case_21 == *cfd_case {
            //             ds_21.truncate(*len)
            //         }
            //     }
            //     Some((None, len)) => ds_21.truncate(*len),
            //     None => (),
            // }
            if let (Some(v_pssn), Some(h_pssn)) = (ds_21.pssn(Band::V), ds_21.pssn(Band::H)) {
                let wfe_rms_21: Vec<_> = ds_21.wfe_rms_iter_10e(-6).into_iter().collect();
                Some((
                    (cfd_case_current.to_string(), v_pssn, h_pssn),
                    if let Some(cfd_case_other) = cfd::Baseline::<R>::find(cfd_case_current) {
                        let path = cfd::Baseline::<R>::path()
                            .ok()?
                            .join(format!("{}", cfd_case_other));
                        let ds_20 =
                            DomeSeeing::load(&path).expect(&format!("failed to load: {:?}", path));
                        if let (Some(v_pssn), Some(h_pssn)) =
                            (ds_20.pssn(Band::V), ds_20.pssn(Band::H))
                        {
                            let wfe_rms_20: Vec<_> =
                                ds_20.wfe_rms_iter_10e(-6).into_iter().collect();
                            make_figure(
                                vec![wfe_rms_21, wfe_rms_20],
                                wfe_labels.as_ref().unwrap_or(&vec!["Y", "2020"]).to_vec(),
                                path_to_case
                                    .join("report")
                                    .join("dome-seeing_wfe-rms.png")
                                    .to_str()
                                    .unwrap(),
                                "WFE RMS [micron]",
                            );
                            make_figure(
                                vec![
                                    ds_21.se_pssn_iter(Band::V),
                                    // ds_21.le_pssn_iter(Band::V),
                                    ds_20.le_pssn_iter(Band::V),
                                ],
                                pssn_labels
                                    .as_ref()
                                    .unwrap_or(&vec!["Y (SE)", "Y (LE)", "2020 (LE)"])
                                    .to_vec(),
                                path_to_case
                                    .join("report")
                                    .join("dome-seeing_v-pssn.png")
                                    .to_str()
                                    .unwrap(),
                                "V PSSn",
                            );
                            make_figure(
                                vec![
                                    ds_21.se_pssn_iter(Band::H),
                                    // ds_21.le_pssn_iter(Band::H),
                                    ds_20.le_pssn_iter(Band::H),
                                ],
                                pssn_labels
                                    .as_ref()
                                    .unwrap_or(&vec!["Y (SE)", "Y (LE)", "2020 (LE)"])
                                    .to_vec(),
                                path_to_case
                                    .join("report")
                                    .join("dome-seeing_h-pssn.png")
                                    .to_str()
                                    .unwrap(),
                                "H PSSn",
                            );
                            Some((cfd_case_other.to_string(), v_pssn, h_pssn))
                        } else {
                            None
                        }
                    } else {
                        make_figure(
                            vec![wfe_rms_21],
                            vec!["Y"],
                            path_to_case
                                .join("report")
                                .join("dome-seeing_wfe-rms.png")
                                .to_str()
                                .unwrap(),
                            "WFE RMS",
                        );
                        make_figure(
                            // vec![ds_21.se_pssn_iter(Band::V), ds_21.le_pssn_iter(Band::V)],
                            vec![ds_21.se_pssn_iter(Band::V)],
                            vec!["Y (SE)", "Y (LE)"],
                            path_to_case
                                .join("report")
                                .join("dome-seeing_v-pssn.png")
                                .to_str()
                                .unwrap(),
                            "V PSSn",
                        );
                        make_figure(
                            // vec![ds_21.se_pssn_iter(Band::H), ds_21.le_pssn_iter(Band::H)],
                            vec![ds_21.se_pssn_iter(Band::H)],
                            vec!["Y (SE)", "Y (LE)"],
                            path_to_case
                                .join("report")
                                .join("dome-seeing_h-pssn.png")
                                .to_str()
                                .unwrap(),
                            "H PSSn",
                        );
                        None
                    },
                ))
            } else {
                None
            }
        })
        .collect();
    // SUMMARY
    /* let (pssn_21, pssn_20): (Vec<_>, Vec<_>) = results.iter().cloned().flatten().unzip();
    let (mut v_21, mut h_21): (Vec<f64>, Vec<f64>) =
        pssn_21.into_iter().map(|(_, x, y)| (x, y)).unzip();
    v_21.sort_by(|a, b| a.partial_cmp(b).unwrap());
    h_21.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let (mut v_20, mut h_20): (Vec<f64>, Vec<f64>) = pssn_20
        .into_iter()
        .flatten()
        .map(|(_, x, y)| (x, y))
        .unzip();
    v_20.sort_by(|a, b| a.partial_cmp(b).unwrap());
    h_20.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut id_20 = 0;
    let mut id_21 = 0;
    results.into_iter().for_each(|res| match res {
        Some(((cfd_case_21, v_pssn_21, h_pssn_21), Some((cfd_case_20, v_pssn_20, h_pssn_20)))) => {
            id_20 += 1;
            id_21 += 1;
            println!(
                "{:02} {:<20}: (V: {:.4},H: {:.4}) || {:02} {:<24}: (V: {:.4},H: {:.4})",
                id_21, cfd_case_21, v_pssn_21, h_pssn_21, id_20, cfd_case_20, v_pssn_20, h_pssn_20
            )
        }
        Some(((cfd_case_21, v_pssn_21, h_pssn_21), None)) => {
            id_21 += 1;
            println!(
                "{:02} {:<20}: (V: {:.4},H: {:.4})",
                id_21, cfd_case_21, v_pssn_21, h_pssn_21
            )
        }
        _ => unimplemented!(),
    });
    let med = |x: &[f64]| {
        let n = x.len();
        if n % 2 == 0 {
            0.5 * (x[n / 2] + x[n / 2 + 1])
        } else {
            x[1 + n / 2]
        }
    };
    println!(
        "CFD 2020 median PSSn V: {:.4}, H: {:.4}",
        med(&v_20),
        med(&h_20)
    );
    println!(
        "CFD Y median PSSn V: {:.4}, H: {:.4}",
        med(&v_21),
        med(&h_21)
    ); */
    Ok(())
}
