use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
use crate::model::*;
/**You should use this struct via [`RommApiClient::get_supported_platforms_api_platforms_supported_get`].

On request success, this will return a [`Vec<PlatformSchema>`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSupportedPlatformsApiPlatformsSupportedGetRequest {}
impl FluentRequest<'_, GetSupportedPlatformsApiPlatformsSupportedGetRequest> {}
impl<'a> ::std::future::IntoFuture
    for FluentRequest<'a, GetSupportedPlatformsApiPlatformsSupportedGetRequest>
{
    type Output = httpclient::InMemoryResult<crate::model::Vec<PlatformSchema>>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/platforms/supported";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Supported Platforms

    Get list of supported platforms endpoint

    Args:
        request (Request): Fastapi Request object

    Returns:
        list[PlatformSchema]: List of supported platforms*/
    pub fn get_supported_platforms_api_platforms_supported_get(
        &self,
    ) -> FluentRequest<'_, GetSupportedPlatformsApiPlatformsSupportedGetRequest> {
        FluentRequest {
            client: self,
            params: GetSupportedPlatformsApiPlatformsSupportedGetRequest {},
        }
    }
}
