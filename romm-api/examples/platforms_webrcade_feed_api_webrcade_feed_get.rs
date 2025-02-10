#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let response = client
        .platforms_webrcade_feed_api_webrcade_feed_get()
        .await
        .unwrap();
    println!("{:#?}", response);
}
