use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::add_collection_api_collections_post`].

On request success, this will return a [`CollectionSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCollectionApiCollectionsPostRequest {}
impl FluentRequest<'_, AddCollectionApiCollectionsPostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, AddCollectionApiCollectionsPostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::CollectionSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/collections";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Add Collection

    Create collection endpoint

    Args:
        request (Request): Fastapi Request object

    Returns:
        CollectionSchema: Just created collection*/
    pub fn add_collection_api_collections_post(
        &self,
    ) -> FluentRequest<'_, AddCollectionApiCollectionsPostRequest> {
        FluentRequest {
            client: self,
            params: AddCollectionApiCollectionsPostRequest {},
        }
    }
}
