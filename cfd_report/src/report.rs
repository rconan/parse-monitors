//! CREATE THE COMPLETE YREPORT

use indicatif::ProgressBar;
use parse_monitors::CFD_YEAR;
use parse_monitors::cfd::CfdCase;
use parse_monitors::{cfd, report, report::Report};
use std::time::Duration;
use std::{sync::Arc, time::Instant};
//use tectonic;
use std::thread::{self, JoinHandle};

use crate::pressure_maps::Config;
use crate::{
    ForcesCli, PREVIOUS_YEAR, PROGRESS, ReportError, ReportOptions, batch_force, dome_seeing,
    opd_maps, pressure_maps,
};

type Result = std::result::Result<(), ReportError>;

fn domeseeing<const Y: u32>(
    cfd_cases: &[CfdCase<Y>],
    parts_base: u8,
    h: &mut Vec<JoinHandle<Result>>,
) -> Result {
    let cases: Arc<[cfd::CfdCase<Y>]> = cfd_cases.into();
    let pb = PROGRESS.add(ProgressBar::new_spinner());
    pb.enable_steady_tick(Duration::from_millis(100));
    h.push(thread::spawn(move || {
        pb.set_message("Generating the WFE RMS & PSSn plots ...");
        dome_seeing::taks::<PREVIOUS_YEAR, _>(&cases)?;
        pb.set_message("Generating OPD maps...");
        opd_maps::task(&cases)?;
        pb.set_message("Building the dome seeing part of the report ...");
        report::DomeSeeingPart::new(1 + parts_base, 0f64)
            .part_with(Some(&cases))
            .map_err(|e| ReportError::DomeSeeing(e))?;
        pb.finish();
        Ok(())
    }));
    Ok(())
}
fn htc<const Y: u32>(cfd_cases: &[CfdCase<Y>], parts_base: u8, h: &mut Vec<JoinHandle<Result>>) {
    let cases: Arc<[cfd::CfdCase<Y>]> = cfd_cases.into();
    let pb = PROGRESS
        .add(ProgressBar::new_spinner().with_message("Building the HTC part of the report ..."));
    pb.enable_steady_tick(Duration::from_millis(100));
    h.push(thread::spawn(move || {
        report::HTC::new(3 + parts_base, 400f64)
            .part_with(Some(&cases))
            .map_err(|e| ReportError::HTC(e))?;
        pb.finish();
        Ok(())
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
    let cases: Arc<[cfd::CfdCase<Y>]> = cfd_cases.into();
    let pb = PROGRESS.add(ProgressBar::new_spinner());
    pb.enable_steady_tick(Duration::from_millis(100));
    h.push(thread::spawn(move || {
        pb.set_message("Generating the forces & moments plots ...");
        batch_force::task(&cases, ForcesCli::all())?;
        pb.set_message("Generating M1 & M2 pressure maps...");
        pressure_maps::task::<geotrans::M1, _>(&cases)?;
        pressure_maps::task::<geotrans::M2, _>(&cases)?;
        pb.set_message("Building the wind loads part of the report ...");
        report::WindLoads::new(2 + parts_base, 400f64)
            .exclude_monitors("floor|enclosure|screen|shutter|M1level")
            .keep_last(400)
            //.show_m12_pressure()
            .part_with(Some(&cases))
            .map_err(|e| ReportError::WindLoads(e))?;
        pb.finish();
        Ok(())
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

    println!(" ... CFD {CFD_YEAR} report build in {}s", now.elapsed().as_secs());

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
