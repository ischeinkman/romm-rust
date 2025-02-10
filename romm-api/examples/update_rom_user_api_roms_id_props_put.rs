#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let id = 1;
    let response = client
        .update_rom_user_api_roms_id_props_put(id)
        .await
        .unwrap();
    println!("{:#?}", response);
}
