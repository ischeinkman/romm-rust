use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::delete_platform_version_api_config_system_versions_fs_slug_delete`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePlatformVersionApiConfigSystemVersionsFsSlugDeleteRequest {
    pub fs_slug: String,
}
impl FluentRequest<'_, DeletePlatformVersionApiConfigSystemVersionsFsSlugDeleteRequest> {}
impl<'a> ::std::future::IntoFuture
    for FluentRequest<'a, DeletePlatformVersionApiConfigSystemVersionsFsSlugDeleteRequest>
{
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/api/config/system/versions/{fs_slug}",
                fs_slug = self.params.fs_slug
            );
            let mut r = self.client.client.delete(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Delete Platform Version

    Delete platform version from the configuration*/
    pub fn delete_platform_version_api_config_system_versions_fs_slug_delete(
        &self,
        fs_slug: &str,
    ) -> FluentRequest<'_, DeletePlatformVersionApiConfigSystemVersionsFsSlugDeleteRequest> {
        FluentRequest {
            client: self,
            params: DeletePlatformVersionApiConfigSystemVersionsFsSlugDeleteRequest {
                fs_slug: fs_slug.to_owned(),
            },
        }
    }
}
