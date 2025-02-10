#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let path = "your path";
    let response = client
        .head_raw_asset_api_raw_assets_path_head(path)
        .await
        .unwrap();
    println!("{:#?}", response);
}
