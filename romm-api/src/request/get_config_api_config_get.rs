use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::get_config_api_config_get`].

On request success, this will return a [`ConfigResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetConfigApiConfigGetRequest {}
impl FluentRequest<'_, GetConfigApiConfigGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetConfigApiConfigGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::ConfigResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/config";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Config

    Get config endpoint

    Returns:
        ConfigResponse: RomM's configuration*/
    pub fn get_config_api_config_get(&self) -> FluentRequest<'_, GetConfigApiConfigGetRequest> {
        FluentRequest {
            client: self,
            params: GetConfigApiConfigGetRequest {},
        }
    }
}
