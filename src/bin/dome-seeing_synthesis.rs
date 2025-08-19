use parse_monitors::{cfd, cfd::BaselineTrait, Band, DomeSeeing};
use rayon::prelude::*;

// MAIN
fn main() -> anyhow::Result<()> {
    let cfd_cases_21 = cfd::Baseline::<2021>::at_zenith(cfd::ZenithAngle::Thirty)
        .into_iter()
        .collect::<Vec<cfd::CfdCase<2021>>>();
    let results: Vec<_> = cfd_cases_21
        .into_par_iter()
        .map(|cfd_case_21| {
            let path_to_case = cfd::Baseline::<2021>::path()
                .unwrap()
                .join(format!("{}", cfd_case_21));
            let ds_21 = DomeSeeing::load(path_to_case.clone()).unwrap();
            if let (Some(v_pssn), Some(h_pssn)) = (ds_21.pssn(Band::V), ds_21.pssn(Band::H)) {
                let wfe_rms =
                    1e9 * (ds_21.wfe_rms().map(|x| x * x).sum::<f64>() / ds_21.len() as f64).sqrt();
                Some((
                    (cfd_case_21.clone(), wfe_rms, v_pssn, h_pssn),
                    if let Some(cfd_case_20) = cfd::Baseline::<2020>::find(cfd_case_21) {
                        let ds_20 = DomeSeeing::load(
                            cfd::Baseline::<2020>::path()
                                .unwrap()
                                .join(format!("{}", cfd_case_20)),
                        )
                        .unwrap();
                        if let (Some(v_pssn), Some(h_pssn)) =
                            (ds_20.pssn(Band::V), ds_20.pssn(Band::H))
                        {
                            let wfe_rms = 1e9
                                * (ds_20.wfe_rms().map(|x| x * x).sum::<f64>()
                                    / ds_20.len() as f64)
                                    .sqrt();
                            Some((cfd_case_20, wfe_rms, v_pssn, h_pssn))
                        } else {
                            None
                        }
                    } else {
                        None
                    },
                ))
            } else {
                None
            }
        })
        .collect();
    for result in results {
        match result {
            Some((
                (cfd_case_21, wfe_rms_21, v_pssn_21, h_pssn_21),
                Some((_, wfe_rms_20, v_pssn_20, h_pssn_20)),
            )) => {
                println!(
                    r#" {:} & {:6.0} & {:.4} & {:.4} & {:6.0} & {:.4} & {:.4} \\"#,
                    cfd_case_21.to_latex_string(),
                    wfe_rms_21,
                    v_pssn_21,
                    h_pssn_21,
                    wfe_rms_20,
                    v_pssn_20,
                    h_pssn_20
                )
            }
            Some(((cfd_case_21, wfe_rms_21, v_pssn_21, h_pssn_21), None)) => {
                println!(
                    r#" {:} & {:6.0} & {:.4} & {:.4} \\"#,
                    cfd_case_21.to_latex_string(),
                    wfe_rms_21,
                    v_pssn_21,
                    h_pssn_21,
                )
            }
            _ => unimplemented!(),
        }
    }
    Ok(())
}
