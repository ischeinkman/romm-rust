use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::add_exclusion_api_config_exclude_post`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddExclusionApiConfigExcludePostRequest {}
impl FluentRequest<'_, AddExclusionApiConfigExcludePostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, AddExclusionApiConfigExcludePostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/config/exclude";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Add Exclusion

    Add platform exclusion to the configuration*/
    pub fn add_exclusion_api_config_exclude_post(
        &self,
    ) -> FluentRequest<'_, AddExclusionApiConfigExcludePostRequest> {
        FluentRequest {
            client: self,
            params: AddExclusionApiConfigExcludePostRequest {},
        }
    }
}
