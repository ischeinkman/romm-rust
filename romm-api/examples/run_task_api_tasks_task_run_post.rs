#![allow(unused_imports)]
use romm_api::model::*;
use romm_api::RommApiClient;
#[tokio::main]
async fn main() {
    let client = RommApiClient::from_env();
    let task = "your task";
    let response = client.run_task_api_tasks_task_run_post(task).await.unwrap();
    println!("{:#?}", response);
}
