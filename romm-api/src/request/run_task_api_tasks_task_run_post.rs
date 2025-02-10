use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::run_task_api_tasks_task_run_post`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTaskApiTasksTaskRunPostRequest {
    pub task: String,
}
impl FluentRequest<'_, RunTaskApiTasksTaskRunPostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, RunTaskApiTasksTaskRunPostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/tasks/{task}/run", task = self.params.task);
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Run Task

    Run all tasks endpoint

    Args:
        request (Request): Fastapi Request object
    Returns:
        RunTasksResponse: Standard message response*/
    pub fn run_task_api_tasks_task_run_post(
        &self,
        task: &str,
    ) -> FluentRequest<'_, RunTaskApiTasksTaskRunPostRequest> {
        FluentRequest {
            client: self,
            params: RunTaskApiTasksTaskRunPostRequest {
                task: task.to_owned(),
            },
        }
    }
}
