use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::delete_saves_api_saves_delete_post`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSavesApiSavesDeletePostRequest {}
impl FluentRequest<'_, DeleteSavesApiSavesDeletePostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, DeleteSavesApiSavesDeletePostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/saves/delete";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    ///Delete Saves
    pub fn delete_saves_api_saves_delete_post(
        &self,
    ) -> FluentRequest<'_, DeleteSavesApiSavesDeletePostRequest> {
        FluentRequest {
            client: self,
            params: DeleteSavesApiSavesDeletePostRequest {},
        }
    }
}
