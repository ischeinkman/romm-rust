#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let response = client
        .tinfoil_index_feed_api_tinfoil_feed_get()
        .slug("your slug")
        .await
        .unwrap();
    println!("{:#?}", response);
}
