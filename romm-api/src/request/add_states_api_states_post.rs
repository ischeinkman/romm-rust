use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::add_states_api_states_post`].

On request success, this will return a [`UploadedStatesResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddStatesApiStatesPostRequest {
    pub emulator: Option<serde_json::Value>,
    pub rom_id: i64,
}
impl FluentRequest<'_, AddStatesApiStatesPostRequest> {
    ///Set the value of the emulator field.
    pub fn emulator(mut self, emulator: serde_json::Value) -> Self {
        self.params.emulator = Some(emulator);
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, AddStatesApiStatesPostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::UploadedStatesResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/states";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    ///Add States
    pub fn add_states_api_states_post(
        &self,
        rom_id: i64,
    ) -> FluentRequest<'_, AddStatesApiStatesPostRequest> {
        FluentRequest {
            client: self,
            params: AddStatesApiStatesPostRequest {
                emulator: None,
                rom_id,
            },
        }
    }
}
