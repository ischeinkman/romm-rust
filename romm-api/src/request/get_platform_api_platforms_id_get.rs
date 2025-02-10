use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::get_platform_api_platforms_id_get`].

On request success, this will return a [`PlatformSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPlatformApiPlatformsIdGetRequest {
    pub id: i64,
}
impl FluentRequest<'_, GetPlatformApiPlatformsIdGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetPlatformApiPlatformsIdGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::PlatformSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/platforms/{id}", id = self.params.id);
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Platform

    Get platforms endpoint

    Args:
        request (Request): Fastapi Request object
        id (int, optional): Platform id. Defaults to None.

    Returns:
        PlatformSchema: Platform*/
    pub fn get_platform_api_platforms_id_get(
        &self,
        id: i64,
    ) -> FluentRequest<'_, GetPlatformApiPlatformsIdGetRequest> {
        FluentRequest {
            client: self,
            params: GetPlatformApiPlatformsIdGetRequest { id },
        }
    }
}
