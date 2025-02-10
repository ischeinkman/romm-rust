use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::tinfoil_index_feed_api_tinfoil_feed_get`].

On request success, this will return a [`TinfoilFeedSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinfoilIndexFeedApiTinfoilFeedGetRequest {
    pub slug: Option<String>,
}
impl FluentRequest<'_, TinfoilIndexFeedApiTinfoilFeedGetRequest> {
    ///Set the value of the slug field.
    pub fn slug(mut self, slug: &str) -> Self {
        self.params.slug = Some(slug.to_owned());
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, TinfoilIndexFeedApiTinfoilFeedGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::TinfoilFeedSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/tinfoil/feed";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Tinfoil Index Feed

    Get tinfoil custom index feed endpoint
    https://blawar.github.io/tinfoil/custom_index/

    Args:
        request (Request): Fastapi Request object
        slug (str, optional): Platform slug. Defaults to "switch".

    Returns:
        TinfoilFeedSchema: Tinfoil feed object schema*/
    pub fn tinfoil_index_feed_api_tinfoil_feed_get(
        &self,
    ) -> FluentRequest<'_, TinfoilIndexFeedApiTinfoilFeedGetRequest> {
        FluentRequest {
            client: self,
            params: TinfoilIndexFeedApiTinfoilFeedGetRequest { slug: None },
        }
    }
}
