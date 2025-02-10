#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let file_name = "your file name";
    let id = 1;
    let response = client
        .head_rom_content_api_roms_id_content_file_name_head(file_name, id)
        .files(serde_json::json!({}))
        .await
        .unwrap();
    println!("{:#?}", response);
}
