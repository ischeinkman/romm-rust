use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::update_rom_user_api_roms_id_props_put`].

On request success, this will return a [`RomUserSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRomUserApiRomsIdPropsPutRequest {
    pub id: i64,
}
impl FluentRequest<'_, UpdateRomUserApiRomsIdPropsPutRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, UpdateRomUserApiRomsIdPropsPutRequest> {
    type Output = httpclient::InMemoryResult<crate::model::RomUserSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/roms/{id}/props", id = self.params.id);
            let mut r = self.client.client.put(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    ///Update Rom User
    pub fn update_rom_user_api_roms_id_props_put(
        &self,
        id: i64,
    ) -> FluentRequest<'_, UpdateRomUserApiRomsIdPropsPutRequest> {
        FluentRequest {
            client: self,
            params: UpdateRomUserApiRomsIdPropsPutRequest { id },
        }
    }
}
