use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::delete_states_api_states_delete_post`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteStatesApiStatesDeletePostRequest {}
impl FluentRequest<'_, DeleteStatesApiStatesDeletePostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, DeleteStatesApiStatesDeletePostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/states/delete";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    ///Delete States
    pub fn delete_states_api_states_delete_post(
        &self,
    ) -> FluentRequest<'_, DeleteStatesApiStatesDeletePostRequest> {
        FluentRequest {
            client: self,
            params: DeleteStatesApiStatesDeletePostRequest {},
        }
    }
}
