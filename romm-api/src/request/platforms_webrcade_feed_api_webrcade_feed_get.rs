use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::platforms_webrcade_feed_api_webrcade_feed_get`].

On request success, this will return a [`WebrcadeFeedSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformsWebrcadeFeedApiWebrcadeFeedGetRequest {}
impl FluentRequest<'_, PlatformsWebrcadeFeedApiWebrcadeFeedGetRequest> {}
impl<'a> ::std::future::IntoFuture
    for FluentRequest<'a, PlatformsWebrcadeFeedApiWebrcadeFeedGetRequest>
{
    type Output = httpclient::InMemoryResult<crate::model::WebrcadeFeedSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/webrcade/feed";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Platforms Webrcade Feed

    Get webrcade feed endpoint
    https://docs.webrcade.com/feeds/format/

    Args:
        request (Request): Fastapi Request object

    Returns:
        WebrcadeFeedSchema: Webrcade feed object schema*/
    pub fn platforms_webrcade_feed_api_webrcade_feed_get(
        &self,
    ) -> FluentRequest<'_, PlatformsWebrcadeFeedApiWebrcadeFeedGetRequest> {
        FluentRequest {
            client: self,
            params: PlatformsWebrcadeFeedApiWebrcadeFeedGetRequest {},
        }
    }
}
