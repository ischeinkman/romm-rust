#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let response = client
        .get_supported_platforms_api_platforms_supported_get()
        .await
        .unwrap();
    println!("{:#?}", response);
}
