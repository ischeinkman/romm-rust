#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let id = 1;
    let response = client.get_user_api_users_id_get(id).await.unwrap();
    println!("{:#?}", response);
}
