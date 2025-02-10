use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::get_rom_api_roms_id_get`].

On request success, this will return a [`DetailedRomSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRomApiRomsIdGetRequest {
    pub id: i64,
}
impl FluentRequest<'_, GetRomApiRomsIdGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetRomApiRomsIdGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::DetailedRomSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/roms/{id}", id = self.params.id);
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Rom

    Get rom endpoint

    Args:
        request (Request): Fastapi Request object
        id (int): Rom internal id

    Returns:
        DetailedRomSchema: Rom stored in the database*/
    pub fn get_rom_api_roms_id_get(&self, id: i64) -> FluentRequest<'_, GetRomApiRomsIdGetRequest> {
        FluentRequest {
            client: self,
            params: GetRomApiRomsIdGetRequest { id },
        }
    }
}
