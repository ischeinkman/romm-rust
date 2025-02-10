#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let id = 1;
    let response = client.get_firmware_api_firmware_id_get(id).await.unwrap();
    println!("{:#?}", response);
}
