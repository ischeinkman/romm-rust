use crate::FluentRequest;
use httpclient::{InMemoryResponseExt, Method};
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::head_rom_content_api_roms_id_content_file_name_head`].

On request success, this will return a [`HeadRomContentApiRomsIdContentFileNameHeadResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadRomContentApiRomsIdContentFileNameHeadRequest {
    pub file_name: String,
    pub files: Option<serde_json::Value>,
    pub id: i64,
}
impl FluentRequest<'_, HeadRomContentApiRomsIdContentFileNameHeadRequest> {
    ///Set the value of the files field.
    pub fn files(mut self, files: serde_json::Value) -> Self {
        self.params.files = Some(files);
        self
    }
}
impl<'a> ::std::future::IntoFuture
    for FluentRequest<'a, HeadRomContentApiRomsIdContentFileNameHeadRequest>
{
    type Output = httpclient::InMemoryResult<
        crate::model::HeadRomContentApiRomsIdContentFileNameHeadResponse,
    >;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/api/roms/{id}/content/{file_name}",
                file_name = self.params.file_name,
                id = self.params.id
            );
            let mut r = self.client.client.request(Method::HEAD, url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Head Rom Content

    Head rom content endpoint

    Args:
        request (Request): Fastapi Request object
        id (int): Rom internal id
        file_name (str): Required due to a bug in emulatorjs

    Returns:
        FileResponse: Returns the response with headers*/
    pub fn head_rom_content_api_roms_id_content_file_name_head(
        &self,
        file_name: &str,
        id: i64,
    ) -> FluentRequest<'_, HeadRomContentApiRomsIdContentFileNameHeadRequest> {
        FluentRequest {
            client: self,
            params: HeadRomContentApiRomsIdContentFileNameHeadRequest {
                file_name: file_name.to_owned(),
                files: None,
                id,
            },
        }
    }
}
