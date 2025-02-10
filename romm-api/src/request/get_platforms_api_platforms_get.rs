use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
use crate::model::*;
/**You should use this struct via [`RommApiClient::get_platforms_api_platforms_get`].

On request success, this will return a [`Vec<PlatformSchema>`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPlatformsApiPlatformsGetRequest {}
impl FluentRequest<'_, GetPlatformsApiPlatformsGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetPlatformsApiPlatformsGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::Vec<PlatformSchema>>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/platforms";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Platforms

    Get platforms endpoint

    Args:
        request (Request): Fastapi Request object
        id (int, optional): Platform id. Defaults to None.

    Returns:
        list[PlatformSchema]: List of platforms*/
    pub fn get_platforms_api_platforms_get(
        &self,
    ) -> FluentRequest<'_, GetPlatformsApiPlatformsGetRequest> {
        FluentRequest {
            client: self,
            params: GetPlatformsApiPlatformsGetRequest {},
        }
    }
}
