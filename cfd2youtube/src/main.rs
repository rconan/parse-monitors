use std::default::Default;
use youtube3::{hyper, hyper_rustls, oauth2, YouTube,api::Video};
// use youtube3::{Error, Result};
use std::fs::File;

#[tokio::main]
async fn main() {
        let secret = oauth2::read_application_secret("client_secrets.json")
        .await
        .expect("clientsecret.json");
    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .build()
    .await
    .unwrap();
    let mut hub = YouTube::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()), auth);
let mut req = Video::default();
let rb = hub.videos().insert(req).upload(File::open("/home/ubuntu/Notebooks/active_optics.mp4").unwrap(), "application/octet-stream".parse().unwrap()).await;;
;}
