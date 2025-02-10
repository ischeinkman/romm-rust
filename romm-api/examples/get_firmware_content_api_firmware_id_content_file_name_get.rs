#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let file_name = "your file name";
    let id = 1;
    let response = client
        .get_firmware_content_api_firmware_id_content_file_name_get(file_name, id)
        .await
        .unwrap();
    println!("{:#?}", response);
}
