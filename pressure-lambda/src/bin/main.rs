use bytes::Bytes;
use bzip2::read::BzDecoder;
use lambda_runtime::{handler_fn, Context, Error};
use pressure_lambda::Pressure;
use s3::Client;
use serde_json::{json, Value};
use std::{io::Read, path::Path};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = handler_fn(func);
    lambda_runtime::run(func).await?;
    /*
        let value = json!({"key": "Baseline2021/Baseline2021/Baseline2021/CASES/zen30az000_OS7/pressures/M1p_M1p_7.000000e+02.csv.bz2"});
        let response = func(value, Context::default()).await?;
        println!("{}", response);
    */
    Ok(())
}

async fn func(event: Value, _: Context) -> Result<Value, Error> {
    let path = Path::new(event["key"].as_str().unwrap());

    let shared_config = aws_config::from_env().region("us-east-2").load().await;
    let s3 = Client::new(&shared_config);

    let mut csv_pressure = String::new();
    {
        let result = s3
            .get_object()
            .bucket("gmto.starccm")
            .key(path.to_str().unwrap())
            .response_content_type("text/csv")
            .send()
            .await?;
        let data: Bytes = result.body.collect().await.map(|data| data.into_bytes())?;
        BzDecoder::new(data.as_ref()).read_to_string(&mut csv_pressure)?;
    }
    let mut csv_geometry = String::new();
    {
        let result = s3
            .get_object()
            .bucket("gmto.starccm")
            .key(path.with_file_name("M1p.csv.bz2").to_str().unwrap())
            .response_content_type("text/csv")
            .send()
            .await?;
        let data: Bytes = result.body.collect().await.map(|data| data.into_bytes())?;
        BzDecoder::new(data.as_ref()).read_to_string(&mut csv_geometry)?;
    }
    let mut pressures = Pressure::load(csv_pressure, csv_geometry).unwrap();
    let segments_integrated_force: Vec<_> =
        (1..=7).map(|sid| pressures.segment_force(sid)).collect();
    Ok(json!({
        "segments integrated force": segments_integrated_force
    }))
}
