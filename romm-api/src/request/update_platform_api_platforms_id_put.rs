use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::update_platform_api_platforms_id_put`].

On request success, this will return a [`PlatformSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlatformApiPlatformsIdPutRequest {
    pub id: i64,
}
impl FluentRequest<'_, UpdatePlatformApiPlatformsIdPutRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, UpdatePlatformApiPlatformsIdPutRequest> {
    type Output = httpclient::InMemoryResult<crate::model::PlatformSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/platforms/{id}", id = self.params.id);
            let mut r = self.client.client.put(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Update Platform

    Update platform endpoint

    Args:
        request (Request): Fastapi Request object
        id (int): Platform id

    Returns:
        MessageResponse: Standard message response*/
    pub fn update_platform_api_platforms_id_put(
        &self,
        id: i64,
    ) -> FluentRequest<'_, UpdatePlatformApiPlatformsIdPutRequest> {
        FluentRequest {
            client: self,
            params: UpdatePlatformApiPlatformsIdPutRequest { id },
        }
    }
}
