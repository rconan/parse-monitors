//! CREATE THE COMPLETE YREPORT

use parse_monitors::CFD_YEAR;
use parse_monitors::cfd::CfdCase;
use parse_monitors::{cfd, report, report::Report};
use std::{sync::Arc, time::Instant};
//use tectonic;
use std::thread::{self, JoinHandle};

use crate::pressure_maps::Config;
use crate::{
    ForcesCli, PREVIOUS_YEAR, ReportError, ReportOptions, batch_force, dome_seeing, opd_maps,
    pressure_maps,
};

type Result = std::result::Result<(), ReportError>;

fn domeseeing<const Y: u32>(
    cfd_cases: &[CfdCase<Y>],
    parts_base: u8,
    h: &mut Vec<JoinHandle<Result>>,
) -> Result {
    println!("Building the dome seeing part of the report ...");
    dome_seeing::taks::<PREVIOUS_YEAR, _>(&cfd_cases)?;
    opd_maps::task(&cfd_cases)?;
    let cases: Arc<Option<Vec<cfd::CfdCase<Y>>>> = Arc::new(Some(cfd_cases.to_vec()));
    h.push(thread::spawn(move || {
        report::DomeSeeingPart::new(1 + parts_base, 0f64)
            .part_with(cases.as_deref())
            .map_err(|e| e.into())
    }));
    Ok(())
}
fn htc<const Y: u32>(cfd_cases: &[CfdCase<Y>], parts_base: u8, h: &mut Vec<JoinHandle<Result>>) {
    println!("Building the HTC part of the report ...");
    let cases: Arc<Option<Vec<cfd::CfdCase<Y>>>> = Arc::new(Some(cfd_cases.to_vec()));
    h.push(thread::spawn(move || {
        report::HTC::new(3 + parts_base, 400f64)
            .part_with(cases.as_deref())
            .map_err(|e| e.into())
    }));
}
fn windloads<const Y: u32>(
    cfd_cases: &[CfdCase<Y>],
    parts_base: u8,
    h: &mut Vec<JoinHandle<Result>>,
) -> Result
where
    geotrans::M1: Config<CfdCase = cfd::CfdCase<Y>> + Default,
    geotrans::M2: Config<CfdCase = cfd::CfdCase<Y>> + Default,
{
    println!("Building the wind loads part of the report ...");
    batch_force::task(&cfd_cases, ForcesCli::all())?;
    pressure_maps::task::<geotrans::M1, _>(&cfd_cases)?;
    pressure_maps::task::<geotrans::M2, _>(&cfd_cases)?;
    let cases: Arc<Option<Vec<cfd::CfdCase<Y>>>> = Arc::new(Some(cfd_cases.to_vec()));
    h.push(thread::spawn(move || {
        report::WindLoads::new(2 + parts_base, 400f64)
            .exclude_monitors("floor|enclosure|screen|shutter|M1level")
            .keep_last(400)
            //.show_m12_pressure()
            .part_with(cases.as_deref())
            .map_err(|e| e.into())
    }));
    Ok(())
}

pub fn taks(cfd_cases: &[CfdCase<{ CFD_YEAR }>], opt: ReportOptions) -> Result {
    let parts_base = 0;

    let mut tjh = vec![];
    let now = Instant::now();
    match opt {
        ReportOptions::Full => {
            domeseeing(&cfd_cases, parts_base, &mut tjh)?;
            windloads(&cfd_cases, parts_base, &mut tjh)?;
            htc(&cfd_cases, parts_base, &mut tjh);
        }
        ReportOptions::DomeSeeing => {
            domeseeing(&cfd_cases, parts_base, &mut tjh)?;
        }
        ReportOptions::HTC => {
            htc(&cfd_cases, parts_base, &mut tjh);
        }
        ReportOptions::WindLoads => {
            windloads(&cfd_cases, parts_base, &mut tjh)?;
        }
    };

    for t in tjh {
        t.join().unwrap()?;
    }

    println!(" ... report parts build in {}s", now.elapsed().as_secs());

    /*
        let latex = format!(
            r#"
    \documentclass{{report}}
    \usepackage[colorlinks=true,linkcolor=blue]{{hyperref}}\usepackage{{graphicx}}
    \usepackage{{booktabs}}
    \usepackage{{longtable}}

    \addtolength{{\textwidth}}{{3cm}}
    \addtolength{{\headheight}}{{5mm}}
    \addtolength{{\evensidemargin}}{{-2cm}}
    \addtolength{{\oddsidemargin}}{{-1cm}}

    \setcounter{{tocdepth}}{{3}}

    \title{{GMT Observatory {} Computational Fluid Dynamics Census}}
    \author{{R. Conan, K. Vogiatzis, H. Fitzpatrick}}
    \date{{{:?}}}

    \begin{{document}}
    \maketitle
    \tableofcontents
    \listoffigures
    \listoftables

    \part{{Dome Seeing}}

    \include{{report/part1.chapter1}}
    \include{{report/part1.chapter2}}
    \include{{report/part1.chapter3}}

    \part{{Wind Loads}}

    \include{{report/part2.chapter1}}
    \include{{report/part2.chapter2}}
    \include{{report/part2.chapter3}}

    \part{{Heat Transfer Coefficients}}

    \include{{report/part3.chapter1}}
    \include{{report/part3.chapter2}}
    \include{{report/part3.chapter3}}

    \end{{document}}
    "#,
            Y,
            &Local::now().to_rfc2822(),
        );

        let now = Instant::now();
        println!("Compiling the report ...");
        let pdf_data: Vec<u8> = tectonic::latex_to_pdf(latex).expect("processing failed");
        let mut doc = File::create(format!("report/gmto.cfd{}.pdf", Y))?;
        doc.write_all(&pdf_data)?;
        println!(" ... report compiled in {}s", now.elapsed().as_secs());
    */
    Ok(())
}
