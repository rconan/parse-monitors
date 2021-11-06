use bytes::Bytes;
use bzip2::read::BzDecoder;
use s3::Client;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = get_user_data().await?;
    println!("data: {}", data);
    Ok(())
}

async fn get_user_data() -> Result<String, Box<dyn std::error::Error>> {
    // calculate key of s3 object with the hash above
    let key = "data.csv.bz2".to_string();

    // use s3 API to get this object from s3
    let shared_config = aws_config::from_env().region("us-east-2").load().await;
    let s3 = Client::new(&shared_config);

    let result = s3
        .get_object()
        .bucket("gmto.starccm")
        .key(key)
        .response_content_type("text/csv")
        .send()
        .await?;

    let data: Bytes = result.body.collect().await.map(|data| data.into_bytes())?;
    let mut contents = String::new();
    BzDecoder::new(data.as_ref()).read_to_string(&mut contents)?;

    Ok(contents)
}
