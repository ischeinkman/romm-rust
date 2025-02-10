#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let rom_id = 1;
    let response = client
        .search_rom_api_search_roms_get(rom_id)
        .search_by("your search by")
        .search_term(serde_json::json!({}))
        .await
        .unwrap();
    println!("{:#?}", response);
}
