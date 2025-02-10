use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::get_collection_api_collections_id_get`].

On request success, this will return a [`CollectionSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCollectionApiCollectionsIdGetRequest {
    pub id: i64,
}
impl FluentRequest<'_, GetCollectionApiCollectionsIdGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetCollectionApiCollectionsIdGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::CollectionSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/collections/{id}", id = self.params.id);
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Collection

    Get collections endpoint

    Args:
        request (Request): Fastapi Request object
        id (int, optional): Collection id. Defaults to None.

    Returns:
        CollectionSchema: Collection*/
    pub fn get_collection_api_collections_id_get(
        &self,
        id: i64,
    ) -> FluentRequest<'_, GetCollectionApiCollectionsIdGetRequest> {
        FluentRequest {
            client: self,
            params: GetCollectionApiCollectionsIdGetRequest { id },
        }
    }
}
