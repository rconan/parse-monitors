use std::time::Instant;
use std::{fs::File, io::Read, path::Path};

use regex::Regex;

use crate::{Exertion, Monitors, MonitorsError};

use super::MonitorsLoader;

type Result<T> = std::result::Result<T, MonitorsError>;

impl<const Y: u32> MonitorsLoader<Y> {
    #[cfg(feature = "bzip2")]
    fn decompress(&self) -> Result<String> {
        let mut contents = String::new();
        let data_path = Path::new(&self.path).with_extension("csv.bz2");
        let csv_file =
            File::open(&data_path).map_err(|e| MonitorsError::Io(e, data_path.clone()))?;

        log::info!("Loading {:?}...", csv_file);
        let buf = std::io::BufReader::new(csv_file);
        let mut bz2 = bzip2::bufread::BzDecoder::new(buf);
        bz2.read_to_string(&mut contents)
            .map_err(|e| MonitorsError::Io(e, data_path))?;
        Ok(contents)
    }
    #[cfg(not(feature = "bzip2"))]
    fn decompress(&self) -> Result<String> {
        use flate2::read::GzDecoder;

        use crate::MonitorsError;

        let mut contents = String::new();
        let data_path = Path::new(&self.path).with_extension("csv.z");
        log::info!("Loading {:?}...", data_path);
        let csv_file =
            File::open(&data_path).map_err(|e| MonitorsError::Io(e, data_path.clone()))?;
        let mut gz = GzDecoder::new(csv_file);
        gz.read_to_string(&mut contents)
            .map_err(|e| MonitorsError::Io(e, data_path))?;
        Ok(contents)
    }
    pub fn load(self) -> Result<Monitors> {
        let now = Instant::now();
        let contents = self.decompress()?;
        let mut rdr = csv::Reader::from_reader(contents.as_bytes());

        let headers: Vec<_> = {
            let headers = rdr.headers()?;
            //headers.iter().take(20).for_each(|h| println!("{}", h));
            headers.into_iter().map(|h| h.to_string()).collect()
        };
        if Y == 2025 {
            headers
                .iter()
                .find(|h| h.contains("M1c_"))
                .ok_or(MonitorsError::YearMismatch(2021, Y))?;
        }

        let re_htc = Regex::new(
            r"(\w+) Monitor: Surface Average of Heat Transfer Coefficient \(W/m\^2-K\)",
        )?;
        //Cabs_X Monitor 2: Force (N)
        let re_force = Regex::new(r"(.+)_([XYZ]) Monitor(?:: Force)? \(N\)")?;
        let re_moment = Regex::new(r"(.+)Mom_([XYZ]) Monitor(?:: Moment)? \(N-m\)")?;

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
                // HTC
                if let Some(capts) = re_htc.captures(header) {
                    let key = capts.get(1).unwrap().as_str().to_owned();
                    let value = data.parse::<f64>()?;
                    monitors
                        .heat_transfer_coefficients
                        .entry(key)
                        .or_insert_with(Vec::new)
                        .push(value.abs());
                }
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
                        "X" => match exertion.force.x {
                            Some(_) => exertions.push(Exertion::from_force_x(value)),
                            None => {
                                exertion.force.x = Some(value);
                            }
                        },
                        "Y" => match exertion.force.y {
                            Some(_) => exertions.push(Exertion::from_force_y(value)),
                            None => {
                                exertion.force.y = Some(value);
                            }
                        },
                        "Z" => match exertion.force.z {
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
                        "X" => match exertion.moment.x {
                            Some(_) => exertions.push(Exertion::from_moment_x(value)),
                            None => {
                                exertion.moment.x = Some(value);
                            }
                        },
                        "Y" => match exertion.moment.y {
                            Some(_) => exertions.push(Exertion::from_moment_y(value)),
                            None => {
                                exertion.moment.y = Some(value);
                            }
                        },
                        "Z" => match exertion.moment.z {
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
