#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let exclusion_type = "your exclusion type";
    let exclusion_value = "your exclusion value";
    let response = client
        .delete_exclusion_api_config_exclude_exclusion_type_exclusion_value_delete(
            exclusion_type,
            exclusion_value,
        )
        .await
        .unwrap();
    println!("{:#?}", response);
}
