use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
use crate::model::*;
/**You should use this struct via [`RommApiClient::get_collections_api_collections_get`].

On request success, this will return a [`Vec<CollectionSchema>`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCollectionsApiCollectionsGetRequest {}
impl FluentRequest<'_, GetCollectionsApiCollectionsGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetCollectionsApiCollectionsGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::Vec<CollectionSchema>>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/collections";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Collections

    Get collections endpoint

    Args:
        request (Request): Fastapi Request object
        id (int, optional): Collection id. Defaults to None.

    Returns:
        list[CollectionSchema]: List of collections*/
    pub fn get_collections_api_collections_get(
        &self,
    ) -> FluentRequest<'_, GetCollectionsApiCollectionsGetRequest> {
        FluentRequest {
            client: self,
            params: GetCollectionsApiCollectionsGetRequest {},
        }
    }
}
