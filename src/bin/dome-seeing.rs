use parse_monitors::{cfd, Band, DomeSeeing};

fn main() {
    for cfd_case_21 in cfd::Baseline::<2021>::default().into_iter() {
        let ds = DomeSeeing::load(cfd::Baseline::<2021>::path().join(format!("{}", cfd_case_21)))
            .unwrap();
        if let (Some(v_pssn), Some(h_pssn)) = (ds.pssn(Band::V), ds.pssn(Band::H)) {
            print!(
                "{:<24}: (V: {:.4},H: {:.4})",
                format!("{}", cfd_case_21.clone()),
                v_pssn,
                h_pssn
            );
            if let Some(cfd_case_20) = cfd::Baseline::<2020>::find(cfd_case_21.clone()) {
                let ds = DomeSeeing::load(
                    cfd::Baseline::<2020>::path().join(format!("{}", cfd_case_20)),
                )
                .unwrap();
                if let (Some(v_pssn), Some(h_pssn)) = (ds.pssn(Band::V), ds.pssn(Band::H)) {
                    println!(
                        " | {:<24}: (V: {:.4},H: {:.4})",
                        format!("{}", cfd_case_20),
                        v_pssn,
                        h_pssn
                    )
                } else {
                    println!("")
                }
            } else {
                println!("")
            }
        }
    }
}
