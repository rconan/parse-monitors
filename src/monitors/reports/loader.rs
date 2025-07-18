use std::path::Path;

#[cfg(feature = "2020")]
mod y2020;
#[cfg(any(feature = "2021", feature = "2025"))]
mod y2021_25;

pub struct MonitorsLoader<const YEAR: u32> {
    pub(crate) path: String,
    pub(crate) time_range: (f64, f64),
    pub(crate) header_regex: String,
    pub(crate) header_exclude_regex: Option<String>,
}
impl<const YEAR: u32> Default for MonitorsLoader<YEAR> {
    fn default() -> Self {
        Self {
            path: String::from("monitors.csv"),
            time_range: (0f64, f64::INFINITY),
            header_regex: String::from(r"\w+"),
            header_exclude_regex: None,
        }
    }
}
impl<const YEAR: u32> MonitorsLoader<YEAR> {
    pub fn data_path<S: AsRef<Path> + std::convert::AsRef<std::ffi::OsStr>>(
        self,
        data_path: S,
    ) -> Self {
        let path = Path::new(&data_path).join("monitors.csv");
        Self {
            path: path.to_str().unwrap().to_owned(),
            ..self
        }
    }
    pub fn start_time(self, time: f64) -> Self {
        Self {
            time_range: (time, self.time_range.1),
            ..self
        }
    }
    pub fn end_time(self, time: f64) -> Self {
        Self {
            time_range: (self.time_range.0, time),
            ..self
        }
    }
    pub fn header_filter<S: Into<String>>(self, header_regex: S) -> Self {
        Self {
            header_regex: header_regex.into(),
            ..self
        }
    }
    pub fn exclude_filter<S: Into<String>>(self, header_exclude_regex: S) -> Self {
        Self {
            header_exclude_regex: Some(header_exclude_regex.into()),
            ..self
        }
    }
}
