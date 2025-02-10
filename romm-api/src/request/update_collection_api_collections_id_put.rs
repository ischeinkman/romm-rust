use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::update_collection_api_collections_id_put`].

On request success, this will return a [`CollectionSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCollectionApiCollectionsIdPutRequest {
    pub id: i64,
    pub is_public: Option<serde_json::Value>,
    pub remove_cover: Option<bool>,
}
impl FluentRequest<'_, UpdateCollectionApiCollectionsIdPutRequest> {
    ///Set the value of the is_public field.
    pub fn is_public(mut self, is_public: serde_json::Value) -> Self {
        self.params.is_public = Some(is_public);
        self
    }
    ///Set the value of the remove_cover field.
    pub fn remove_cover(mut self, remove_cover: bool) -> Self {
        self.params.remove_cover = Some(remove_cover);
        self
    }
}
impl<'a> ::std::future::IntoFuture
    for FluentRequest<'a, UpdateCollectionApiCollectionsIdPutRequest>
{
    type Output = httpclient::InMemoryResult<crate::model::CollectionSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/collections/{id}", id = self.params.id);
            let mut r = self.client.client.put(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Update Collection

    Update collection endpoint

    Args:
        request (Request): Fastapi Request object

    Returns:
        MessageResponse: Standard message response*/
    pub fn update_collection_api_collections_id_put(
        &self,
        id: i64,
    ) -> FluentRequest<'_, UpdateCollectionApiCollectionsIdPutRequest> {
        FluentRequest {
            client: self,
            params: UpdateCollectionApiCollectionsIdPutRequest {
                id,
                is_public: None,
                remove_cover: None,
            },
        }
    }
}
