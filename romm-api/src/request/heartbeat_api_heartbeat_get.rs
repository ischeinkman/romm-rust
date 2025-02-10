use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::heartbeat_api_heartbeat_get`].

On request success, this will return a [`HeartbeatResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatApiHeartbeatGetRequest {}
impl FluentRequest<'_, HeartbeatApiHeartbeatGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, HeartbeatApiHeartbeatGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::HeartbeatResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/heartbeat";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Heartbeat

    Endpoint to set the CSRF token in cache and return all the basic RomM config

    Returns:
        HeartbeatReturn: TypedDict structure with all the defined values in the HeartbeatReturn class.*/
    pub fn heartbeat_api_heartbeat_get(
        &self,
    ) -> FluentRequest<'_, HeartbeatApiHeartbeatGetRequest> {
        FluentRequest {
            client: self,
            params: HeartbeatApiHeartbeatGetRequest {},
        }
    }
}
