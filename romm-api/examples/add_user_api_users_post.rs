#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::request::add_user_api_users_post::AddUserApiUsersPostRequired;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let email = "your email";
    let password = "your password";
    let role = "your role";
    let username = "your username";
    let response = client
        .add_user_api_users_post(AddUserApiUsersPostRequired {
            email,
            password,
            role,
            username,
        })
        .await
        .unwrap();
    println!("{:#?}", response);
}
