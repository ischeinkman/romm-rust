use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::get_rom_content_api_roms_id_content_file_name_get`].

On request success, this will return a [`GetRomContentApiRomsIdContentFileNameGetResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRomContentApiRomsIdContentFileNameGetRequest {
    pub file_name: String,
    pub files: Option<serde_json::Value>,
    pub id: i64,
}
impl FluentRequest<'_, GetRomContentApiRomsIdContentFileNameGetRequest> {
    ///Set the value of the files field.
    pub fn files(mut self, files: serde_json::Value) -> Self {
        self.params.files = Some(files);
        self
    }
}
impl<'a> ::std::future::IntoFuture
    for FluentRequest<'a, GetRomContentApiRomsIdContentFileNameGetRequest>
{
    type Output =
        httpclient::InMemoryResult<crate::model::GetRomContentApiRomsIdContentFileNameGetResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/api/roms/{id}/content/{file_name}",
                file_name = self.params.file_name,
                id = self.params.id
            );
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Rom Content

    Download rom endpoint (one single file or multiple zipped files for multi-part roms)

    Args:
        request (Request): Fastapi Request object
        id (int): Rom internal id
        files (Annotated[list[str]  |  None, Query, optional): List of files to download for multi-part roms. Defaults to None.

    Returns:
        FileResponse: Returns one file for single file roms

    Yields:
        ZipResponse: Returns a response for nginx to serve a Zip file for multi-part roms*/
    pub fn get_rom_content_api_roms_id_content_file_name_get(
        &self,
        file_name: &str,
        id: i64,
    ) -> FluentRequest<'_, GetRomContentApiRomsIdContentFileNameGetRequest> {
        FluentRequest {
            client: self,
            params: GetRomContentApiRomsIdContentFileNameGetRequest {
                file_name: file_name.to_owned(),
                files: None,
                id,
            },
        }
    }
}
