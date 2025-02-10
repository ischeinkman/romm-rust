#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let response = client
        .get_roms_api_roms_get()
        .collection_id(serde_json::json!({}))
        .limit(serde_json::json!({}))
        .offset(serde_json::json!({}))
        .order_by("your order by")
        .order_dir("your order dir")
        .platform_id(serde_json::json!({}))
        .search_term("your search term")
        .await
        .unwrap();
    println!("{:#?}", response);
}
