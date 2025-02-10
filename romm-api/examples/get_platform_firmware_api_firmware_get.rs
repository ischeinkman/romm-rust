#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let response = client
        .get_platform_firmware_api_firmware_get()
        .platform_id(serde_json::json!({}))
        .await
        .unwrap();
    println!("{:#?}", response);
}
