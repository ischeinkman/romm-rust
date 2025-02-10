#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let rom_id = 1;
    let response = client
        .add_screenshots_api_screenshots_post(rom_id)
        .await
        .unwrap();
    println!("{:#?}", response);
}
