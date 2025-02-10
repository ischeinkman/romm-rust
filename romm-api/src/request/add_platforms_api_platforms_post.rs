use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::add_platforms_api_platforms_post`].

On request success, this will return a [`PlatformSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPlatformsApiPlatformsPostRequest {}
impl FluentRequest<'_, AddPlatformsApiPlatformsPostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, AddPlatformsApiPlatformsPostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::PlatformSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/platforms";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Add Platforms

    Create platform endpoint

    Args:
        request (Request): Fastapi Request object

    Returns:
        PlatformSchema: Just created platform*/
    pub fn add_platforms_api_platforms_post(
        &self,
    ) -> FluentRequest<'_, AddPlatformsApiPlatformsPostRequest> {
        FluentRequest {
            client: self,
            params: AddPlatformsApiPlatformsPostRequest {},
        }
    }
}
