#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let id = 1;
    let response = client
        .update_collection_api_collections_id_put(id)
        .is_public(serde_json::json!({}))
        .remove_cover(true)
        .await
        .unwrap();
    println!("{:#?}", response);
}
