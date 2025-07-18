use std::{path::Path, time::Instant};

use regex::Regex;

use crate::{Exertion, Monitors, MonitorsError};

use super::MonitorsLoader;

type Result<T> = std::result::Result<T, MonitorsError>;

impl MonitorsLoader<2020> {
    pub fn load(self) -> Result<Monitors> {
        let csv_file = Path::new(&self.path).with_file_name("monitors-2020.csv");
        log::info!("Loading {:?}...", csv_file);
        let now = Instant::now();
        let mut rdr = csv::Reader::from_path(csv_file)?;

        let headers: Vec<_> = {
            let headers = rdr.headers()?;
            headers
                .into_iter()
                .map(|x| x.split_whitespace().collect::<Vec<&str>>().join(""))
                .collect()
        };

        let re_force = Regex::new(r"Force(\w+)([xyz])Monitor:Force\(N\)").unwrap();
        let re_moment = Regex::new(r"Moment(\w+)([xyz])Monitor:Moment\(N-m\)").unwrap();

        let re_header = Regex::new(&self.header_regex)?;
        let re_x_header = if let Some(re) = self.header_exclude_regex {
            Some(Regex::new(&re)?)
        } else {
            None
        };

        let mut monitors = Monitors::default();

        for result in rdr.records() {
            let record = result?;
            let time = record.iter().next().unwrap().parse::<f64>()?;
            if time < self.time_range.0 - 1. / 40. || time > self.time_range.1 + 1. / 40. {
                continue;
            };
            monitors.time.push(time);
            for (data, header) in record.iter().skip(1).zip(headers.iter().skip(1)).filter(
                |(_, h)| match &re_x_header {
                    Some(re_x_header) => re_header.is_match(h) && !re_x_header.is_match(h),
                    None => re_header.is_match(h),
                },
            ) {
                // FORCE
                if let Some(capts) = re_force.captures(header) {
                    let key = capts.get(1).unwrap().as_str().to_owned();
                    let value = data.parse::<f64>()?;
                    let exertions = monitors
                        .forces_and_moments
                        .entry(key)
                        .or_insert(vec![Exertion::default()]);
                    let exertion = exertions.last_mut().unwrap();
                    match capts.get(2).unwrap().as_str() {
                        "x" => match exertion.force.x {
                            Some(_) => exertions.push(Exertion::from_force_x(value)),
                            None => {
                                exertion.force.x = Some(value);
                            }
                        },
                        "y" => match exertion.force.y {
                            Some(_) => exertions.push(Exertion::from_force_y(value)),
                            None => {
                                exertion.force.y = Some(value);
                            }
                        },
                        "z" => match exertion.force.z {
                            Some(_) => exertions.push(Exertion::from_force_z(value)),
                            None => {
                                exertion.force.z = Some(value);
                            }
                        },
                        &_ => (),
                    };
                }
                // MOMENT
                if let Some(capts) = re_moment.captures(header) {
                    let key = capts
                        .get(1)
                        .unwrap()
                        .as_str()
                        .trim_end_matches('_')
                        .to_owned();
                    let value = data.parse::<f64>()?;
                    let exertions = monitors
                        .forces_and_moments
                        .entry(key)
                        .or_insert(vec![Exertion::default()]);
                    let exertion = exertions.last_mut().unwrap();
                    match capts.get(2).unwrap().as_str() {
                        "x" => match exertion.moment.x {
                            Some(_) => exertions.push(Exertion::from_moment_x(value)),
                            None => {
                                exertion.moment.x = Some(value);
                            }
                        },
                        "y" => match exertion.moment.y {
                            Some(_) => exertions.push(Exertion::from_moment_y(value)),
                            None => {
                                exertion.moment.y = Some(value);
                            }
                        },
                        "z" => match exertion.moment.z {
                            Some(_) => exertions.push(Exertion::from_moment_z(value)),
                            None => {
                                exertion.moment.z = Some(value);
                            }
                        },
                        &_ => (),
                    };
                }
            }
        }
        log::info!("... loaded in {:}s", now.elapsed().as_secs());
        Ok(monitors)
    }
}
