#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let id = 1;
    let response = client
        .update_rom_api_roms_id_put(id)
        .remove_cover(true)
        .rename_as_source(true)
        .unmatch_metadata(true)
        .await
        .unwrap();
    println!("{:#?}", response);
}
