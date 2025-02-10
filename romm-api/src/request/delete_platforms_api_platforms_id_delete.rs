use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::delete_platforms_api_platforms_id_delete`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePlatformsApiPlatformsIdDeleteRequest {
    pub id: i64,
}
impl FluentRequest<'_, DeletePlatformsApiPlatformsIdDeleteRequest> {}
impl<'a> ::std::future::IntoFuture
    for FluentRequest<'a, DeletePlatformsApiPlatformsIdDeleteRequest>
{
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/platforms/{id}", id = self.params.id);
            let mut r = self.client.client.delete(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Delete Platforms

    Delete platforms endpoint

    Args:
        request (Request): Fastapi Request object
        {
            "platforms": List of rom's ids to delete
        }

    Raises:
        HTTPException: Platform not found

    Returns:
        MessageResponse: Standard message response*/
    pub fn delete_platforms_api_platforms_id_delete(
        &self,
        id: i64,
    ) -> FluentRequest<'_, DeletePlatformsApiPlatformsIdDeleteRequest> {
        FluentRequest {
            client: self,
            params: DeletePlatformsApiPlatformsIdDeleteRequest { id },
        }
    }
}
