use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::run_tasks_api_tasks_run_post`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTasksApiTasksRunPostRequest {}
impl FluentRequest<'_, RunTasksApiTasksRunPostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, RunTasksApiTasksRunPostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/tasks/run";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Run Tasks

    Run all tasks endpoint

    Args:
        request (Request): Fastapi Request object
    Returns:
        RunTasksResponse: Standard message response*/
    pub fn run_tasks_api_tasks_run_post(
        &self,
    ) -> FluentRequest<'_, RunTasksApiTasksRunPostRequest> {
        FluentRequest {
            client: self,
            params: RunTasksApiTasksRunPostRequest {},
        }
    }
}
