#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let response = client
        .add_platform_version_api_config_system_versions_post()
        .await
        .unwrap();
    println!("{:#?}", response);
}
