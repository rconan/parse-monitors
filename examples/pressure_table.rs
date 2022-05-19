use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct Record {
    #[serde(rename = "Cell Index")]
    index: usize,
}

fn main() -> anyhow::Result<()> {
    let path = Path::new("data")
        .join("pressure-table")
        .join("cell_index_geometry.csv");
    //.join("p_map_table_5.241500e+02.csv");
    let mut rdr = csv::Reader::from_path(path)?;
    let mut idx: Vec<usize> = Vec::with_capacity(1274491);
    for results in rdr.deserialize() {
        let record: Record = results?;
        idx.push(record.index);
    }
    let idt = idx
        .iter()
        .fold(0, |s, i| s + if *i == 1904 { 1 } else { 0 });
    println!("1904: x{}", idt);
    let idx_max = idx.iter().cloned().fold(usize::MIN, usize::max);
    let idx_min = idx.iter().cloned().fold(usize::MAX, usize::min);
    println!("{} indices: {:?}", idx.len(), (idx_min, idx_max));
    Ok(())
}
