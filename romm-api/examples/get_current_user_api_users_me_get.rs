#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let response = client.get_current_user_api_users_me_get().await.unwrap();
    println!("{:#?}", response);
}
