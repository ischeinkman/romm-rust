#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let id = 1;
    let response = client
        .update_user_api_users_id_put(id)
        .email(serde_json::json!({}))
        .enabled(serde_json::json!({}))
        .password(serde_json::json!({}))
        .role(serde_json::json!({}))
        .username(serde_json::json!({}))
        .await
        .unwrap();
    println!("{:#?}", response);
}
