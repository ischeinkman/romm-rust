use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::update_save_api_saves_id_put`].

On request success, this will return a [`SaveSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSaveApiSavesIdPutRequest {
    pub id: i64,
}
impl FluentRequest<'_, UpdateSaveApiSavesIdPutRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, UpdateSaveApiSavesIdPutRequest> {
    type Output = httpclient::InMemoryResult<crate::model::SaveSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/saves/{id}", id = self.params.id);
            let mut r = self.client.client.put(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    ///Update Save
    pub fn update_save_api_saves_id_put(
        &self,
        id: i64,
    ) -> FluentRequest<'_, UpdateSaveApiSavesIdPutRequest> {
        FluentRequest {
            client: self,
            params: UpdateSaveApiSavesIdPutRequest { id },
        }
    }
}
