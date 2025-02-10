#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let response = client
        .search_cover_api_search_cover_get()
        .search_term("your search term")
        .await
        .unwrap();
    println!("{:#?}", response);
}
