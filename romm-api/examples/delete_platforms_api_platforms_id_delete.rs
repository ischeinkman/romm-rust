#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let id = 1;
    let response = client
        .delete_platforms_api_platforms_id_delete(id)
        .await
        .unwrap();
    println!("{:#?}", response);
}
