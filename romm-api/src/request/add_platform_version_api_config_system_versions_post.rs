use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::add_platform_version_api_config_system_versions_post`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPlatformVersionApiConfigSystemVersionsPostRequest {}
impl FluentRequest<'_, AddPlatformVersionApiConfigSystemVersionsPostRequest> {}
impl<'a> ::std::future::IntoFuture
    for FluentRequest<'a, AddPlatformVersionApiConfigSystemVersionsPostRequest>
{
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/config/system/versions";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Add Platform Version

    Add platform version to the configuration*/
    pub fn add_platform_version_api_config_system_versions_post(
        &self,
    ) -> FluentRequest<'_, AddPlatformVersionApiConfigSystemVersionsPostRequest> {
        FluentRequest {
            client: self,
            params: AddPlatformVersionApiConfigSystemVersionsPostRequest {},
        }
    }
}
