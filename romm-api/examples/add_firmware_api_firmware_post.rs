#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let platform_id = 1;
    let response = client
        .add_firmware_api_firmware_post(platform_id)
        .await
        .unwrap();
    println!("{:#?}", response);
}
