use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::add_saves_api_saves_post`].

On request success, this will return a [`UploadedSavesResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSavesApiSavesPostRequest {
    pub emulator: Option<serde_json::Value>,
    pub rom_id: i64,
}
impl FluentRequest<'_, AddSavesApiSavesPostRequest> {
    ///Set the value of the emulator field.
    pub fn emulator(mut self, emulator: serde_json::Value) -> Self {
        self.params.emulator = Some(emulator);
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, AddSavesApiSavesPostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::UploadedSavesResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/saves";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    ///Add Saves
    pub fn add_saves_api_saves_post(
        &self,
        rom_id: i64,
    ) -> FluentRequest<'_, AddSavesApiSavesPostRequest> {
        FluentRequest {
            client: self,
            params: AddSavesApiSavesPostRequest {
                emulator: None,
                rom_id,
            },
        }
    }
}
