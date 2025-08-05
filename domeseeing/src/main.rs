use std::{fs::File, path::Path, time::Instant};

use crseo::{
    Builder, FromBuilder, Gmt, PSSnEstimates, Source,
    pssn::{PSSnBuilder, TelescopeError},
};
use gmt_dos_clients_domeseeing::DomeSeeing;
use parse_monitors::cfd::Baseline;

fn main() -> anyhow::Result<()> {
    Baseline::<2025>::default()
        .into_iter()
        .skip(20)
        .take(5)
        .map(|cfd_case| Path::new("/home/ubuntu/mnt/CASES/").join(&cfd_case.to_string()))
        .for_each(|path| task(&path).expect(&format!("{path:?} failed")));

    Ok(())
}
fn task(cfd_path: &Path) -> anyhow::Result<()> {
    println!("{cfd_path:?}");
    // let cfd_path = Path::new("/home/ubuntu/mnt/issues/optvol_mesh/zen30az000_OS_7ms");
    // let ds0 = parse_monitors::DomeSeeing::load(&cfd_path)?;
    // println!("Dome Seeing entry #1: {:#?}", ds0[0]);

    let mut ds = DomeSeeing::builder(&cfd_path).build()?;
    // println!("{ds}");

    let mut gmt = Gmt::builder().build()?;
    let v_src = Source::builder().band("Vs");
    let mut v_pssn = PSSnBuilder::<TelescopeError>::default()
        .source(v_src.clone())
        .build()?;
    let mut v_src = v_src.build()?;
    let h_src = Source::builder().band("H");
    let mut h_pssn = PSSnBuilder::<TelescopeError>::default()
        .source(h_src.clone())
        .build()?;
    let mut h_src = h_src.build()?;

    let mut record = parse_monitors::DomeSeeing::new();
    let mut data = parse_monitors::Data::default();
    let now = Instant::now();
    let mut time = 0f64;
    while let Some(opd) = ds.next() {
        v_src
            .through(&mut gmt)
            .xpupil()
            .add(opd.as_slice())
            .through(&mut v_pssn);
        h_src
            .through(&mut gmt)
            .xpupil()
            .add(opd.as_slice())
            .through(&mut h_pssn);

        // println!("{:4.0}/{:4.0}", d.wfe_rms[0] * 1e9, src.wfe_rms_10e(-9)[0]);
        // if let Some(v_le_pssn) = d.v_le_pssn {
        //     println!(
        //         "{:.5}/{:.5} -- {:.5}",
        //         d.v_se_pssn,
        //         pssn.estimates()[0],
        //         v_le_pssn
        //     );
        // } else {
        //     println!("{:.5}/{:.5}", d.v_se_pssn, pssn.estimates()[0]);
        // }
        // dbg!(&pssn.estimates());
        data.time = time;
        time += 0.2;
        data.wfe_rms = v_src.wfe_rms();
        data.tip_tilt = v_src.gradients();
        data.segment_tip_tilt = v_src.segment_gradients();
        data.segment_piston = v_src.segment_piston();
        data.v_se_pssn = v_pssn.estimates()[0];
        data.h_se_pssn = h_pssn.estimates()[0];
        record.push(data.clone());
    }
    println!(
        "Record[{}] completed in {}s",
        record.len(),
        now.elapsed().as_secs()
    );

    serde_pickle::to_writer(
        &mut File::create(cfd_path.join("domeseeing_PSSN.rs.pkl"))?,
        &record,
        Default::default(),
    )?;
    Ok(())
}
