use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::delete_collections_api_collections_id_delete`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCollectionsApiCollectionsIdDeleteRequest {
    pub id: i64,
}
impl FluentRequest<'_, DeleteCollectionsApiCollectionsIdDeleteRequest> {}
impl<'a> ::std::future::IntoFuture
    for FluentRequest<'a, DeleteCollectionsApiCollectionsIdDeleteRequest>
{
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/collections/{id}", id = self.params.id);
            let mut r = self.client.client.delete(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Delete Collections

    Delete collections endpoint

    Args:
        request (Request): Fastapi Request object
        {
            "collections": List of rom's ids to delete
        }

    Raises:
        HTTPException: Collection not found

    Returns:
        MessageResponse: Standard message response*/
    pub fn delete_collections_api_collections_id_delete(
        &self,
        id: i64,
    ) -> FluentRequest<'_, DeleteCollectionsApiCollectionsIdDeleteRequest> {
        FluentRequest {
            client: self,
            params: DeleteCollectionsApiCollectionsIdDeleteRequest { id },
        }
    }
}
