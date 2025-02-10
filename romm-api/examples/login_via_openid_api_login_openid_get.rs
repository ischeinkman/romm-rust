#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let response = client
        .login_via_openid_api_login_openid_get()
        .await
        .unwrap();
    println!("{:#?}", response);
}
