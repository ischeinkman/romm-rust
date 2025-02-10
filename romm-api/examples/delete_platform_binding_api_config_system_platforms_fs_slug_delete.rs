#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let fs_slug = "your fs slug";
    let response = client
        .delete_platform_binding_api_config_system_platforms_fs_slug_delete(fs_slug)
        .await
        .unwrap();
    println!("{:#?}", response);
}
