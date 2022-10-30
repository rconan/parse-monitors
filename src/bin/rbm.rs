use parse_monitors::cfd::{Baseline, BaselineTrait};
use std::{fs::File, io::Write};

const CFD_YEAR: u32 = 2021;

fn main() -> anyhow::Result<()> {
    for (k, cfd_case) in Baseline::<CFD_YEAR>::default().into_iter().enumerate() {
        println!("CFD CASE #{:02}: {}", k, cfd_case);
        let path_to_case = Baseline::<CFD_YEAR>::path().join(&cfd_case.to_string());
        let rbm_tables = {
            let table = lom::Table::from_parquet(path_to_case.join("windloading.parquet"))?;
            let mut lom = lom::LOM::builder()
                .table_rigid_body_motions(
                    &table,
                    Some("M1RigidBodyMotions"),
                    Some("M2RigidBodyMotions"),
                )?
                .build()?;
            lom.latex();
            lom.to_string()
        };
        let mut file = File::create(path_to_case.join("report").join("rbm_tables.tex"))?;
        write!(file, "{}", rbm_tables)?;
    }
    Ok(())
}
