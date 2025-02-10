use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::update_state_api_states_id_put`].

On request success, this will return a [`StateSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStateApiStatesIdPutRequest {
    pub id: i64,
}
impl FluentRequest<'_, UpdateStateApiStatesIdPutRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, UpdateStateApiStatesIdPutRequest> {
    type Output = httpclient::InMemoryResult<crate::model::StateSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/states/{id}", id = self.params.id);
            let mut r = self.client.client.put(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    ///Update State
    pub fn update_state_api_states_id_put(
        &self,
        id: i64,
    ) -> FluentRequest<'_, UpdateStateApiStatesIdPutRequest> {
        FluentRequest {
            client: self,
            params: UpdateStateApiStatesIdPutRequest { id },
        }
    }
}
