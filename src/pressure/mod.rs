/*!
# Pressure on the telescope mount and mirror segments

## Examples
Loading telescope mount pressure
```no_run
let telescope =
    Telescope::from_path("data/Telescope_p_telescope_7.000000e+02.csv.z").unwrap();
println!("{telescope}");
```
Converting telescope pressure data into a R-Tree
```no_run
let telescope =
    Telescope::from_path("data/Telescope_p_telescope_7.000000e+02.csv.z").unwrap();
println!("{telescope}");
let rtree = telescope.to_rtree();
let node = rtree.locate_at_point(&[-6.71743755562523, 1.18707466192993, -4.88465284781676]);
assert_eq!(
    node.unwrap().clone(),
    Node {
        pressure: -7.29796930557952,
        area_ijk: [
            1.81847333261638e-06,
            1.84800982152254e-06,
            0.0126174546658134,
            ],
            xyz: [-6.71743755562523, 1.18707466192993, -4.88465284781676],
        }
    );
```
*/

mod mirrors;
pub use mirrors::*;
mod telescope;
use serde::Deserialize;
pub use telescope::*;

#[derive(thiserror::Error, Debug)]
pub enum PressureError {
    #[cfg(feature = "bzip2")]
    #[error("Failed to decompress the file")]
    Decompress(#[from] bzip2::Error),
    #[error("Failed to open the pressure file")]
    Io(#[from] std::io::Error),
    #[error("Failed to deserialize the CSV file")]
    Csv(#[from] csv::Error),
    #[error("Failed to apply geometric transformation")]
    Geotrans(#[from] geotrans::Error),
    #[error("Missing decompression protocol")]
    Decompression,
}
type Result<T> = std::result::Result<T, PressureError>;

#[derive(Deserialize, Debug, PartialEq)]
struct Record {
    #[serde(rename = "Area in TCS[i] (m^2)")]
    area_i: f64,
    #[serde(rename = "Area in TCS[j] (m^2)")]
    area_j: f64,
    #[serde(rename = "Area in TCS[k] (m^2)")]
    area_k: f64,
    //#[serde(rename = "Area: Magnitude (m^2)")]
    //area: f64,
    #[serde(rename = "Pressure (Pa)")]
    pressure: f64,
    #[serde(rename = "X (m)")]
    x: f64,
    #[serde(rename = "Y (m)")]
    y: f64,
    #[serde(rename = "Z (m)")]
    z: f64,
}
