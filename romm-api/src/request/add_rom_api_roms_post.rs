use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::add_rom_api_roms_post`].

On request success, this will return a [`AddRomApiRomsPostResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRomApiRomsPostRequest {}
impl FluentRequest<'_, AddRomApiRomsPostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, AddRomApiRomsPostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::AddRomApiRomsPostResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/roms";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Add Rom

    Upload single rom endpoint

    Args:
        request (Request): Fastapi Request object

    Raises:
        HTTPException: No files were uploaded*/
    pub fn add_rom_api_roms_post(&self) -> FluentRequest<'_, AddRomApiRomsPostRequest> {
        FluentRequest {
            client: self,
            params: AddRomApiRomsPostRequest {},
        }
    }
}
