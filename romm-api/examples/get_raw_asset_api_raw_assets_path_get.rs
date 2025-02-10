#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let path = "your path";
    let response = client
        .get_raw_asset_api_raw_assets_path_get(path)
        .await
        .unwrap();
    println!("{:#?}", response);
}
