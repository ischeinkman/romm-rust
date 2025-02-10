use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::stats_api_stats_get`].

On request success, this will return a [`StatsReturn`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsApiStatsGetRequest {}
impl FluentRequest<'_, StatsApiStatsGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, StatsApiStatsGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::StatsReturn>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/stats";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Stats

    Endpoint to return the current RomM stats

    Returns:
        dict: Dictionary with all the stats*/
    pub fn stats_api_stats_get(&self) -> FluentRequest<'_, StatsApiStatsGetRequest> {
        FluentRequest {
            client: self,
            params: StatsApiStatsGetRequest {},
        }
    }
}
