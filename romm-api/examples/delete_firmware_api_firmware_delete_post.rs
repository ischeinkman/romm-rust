#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let response = client
        .delete_firmware_api_firmware_delete_post()
        .await
        .unwrap();
    println!("{:#?}", response);
}
