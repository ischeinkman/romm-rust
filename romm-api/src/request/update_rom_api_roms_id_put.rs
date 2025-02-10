use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::update_rom_api_roms_id_put`].

On request success, this will return a [`DetailedRomSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRomApiRomsIdPutRequest {
    pub id: i64,
    pub remove_cover: Option<bool>,
    pub rename_as_source: Option<bool>,
    pub unmatch_metadata: Option<bool>,
}
impl FluentRequest<'_, UpdateRomApiRomsIdPutRequest> {
    ///Set the value of the remove_cover field.
    pub fn remove_cover(mut self, remove_cover: bool) -> Self {
        self.params.remove_cover = Some(remove_cover);
        self
    }
    ///Set the value of the rename_as_source field.
    pub fn rename_as_source(mut self, rename_as_source: bool) -> Self {
        self.params.rename_as_source = Some(rename_as_source);
        self
    }
    ///Set the value of the unmatch_metadata field.
    pub fn unmatch_metadata(mut self, unmatch_metadata: bool) -> Self {
        self.params.unmatch_metadata = Some(unmatch_metadata);
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, UpdateRomApiRomsIdPutRequest> {
    type Output = httpclient::InMemoryResult<crate::model::DetailedRomSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/roms/{id}", id = self.params.id);
            let mut r = self.client.client.put(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Update Rom

    Update rom endpoint

    Args:
        request (Request): Fastapi Request object
        id (Rom): Rom internal id
        rename_as_source (bool, optional): Flag to rename rom file as matched IGDB game. Defaults to False.
        artwork (UploadFile, optional): Custom artork to set as cover. Defaults to File(None).
        unmatch_metadata: Remove the metadata matches for this game. Defaults to False.

    Raises:
        HTTPException: If a rom already have that name when enabling the rename_as_source flag

    Returns:
        DetailedRomSchema: Rom stored in the database*/
    pub fn update_rom_api_roms_id_put(
        &self,
        id: i64,
    ) -> FluentRequest<'_, UpdateRomApiRomsIdPutRequest> {
        FluentRequest {
            client: self,
            params: UpdateRomApiRomsIdPutRequest {
                id,
                remove_cover: None,
                rename_as_source: None,
                unmatch_metadata: None,
            },
        }
    }
}
