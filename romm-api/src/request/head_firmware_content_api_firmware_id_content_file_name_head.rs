use crate::FluentRequest;
use httpclient::{InMemoryResponseExt, Method};
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::head_firmware_content_api_firmware_id_content_file_name_head`].

On request success, this will return a [`HeadFirmwareContentApiFirmwareIdContentFileNameHeadResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadFirmwareContentApiFirmwareIdContentFileNameHeadRequest {
    pub file_name: String,
    pub id: i64,
}
impl FluentRequest<'_, HeadFirmwareContentApiFirmwareIdContentFileNameHeadRequest> {}
impl<'a> ::std::future::IntoFuture
    for FluentRequest<'a, HeadFirmwareContentApiFirmwareIdContentFileNameHeadRequest>
{
    type Output = httpclient::InMemoryResult<
        crate::model::HeadFirmwareContentApiFirmwareIdContentFileNameHeadResponse,
    >;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/api/firmware/{id}/content/{file_name}",
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
    /**Head Firmware Content

    Head firmware content endpoint

    Args:
        request (Request): Fastapi Request object
        id (int): Rom internal id
        file_name (str): Required due to a bug in emulatorjs

    Returns:
        FileResponse: Returns the response with headers*/
    pub fn head_firmware_content_api_firmware_id_content_file_name_head(
        &self,
        file_name: &str,
        id: i64,
    ) -> FluentRequest<'_, HeadFirmwareContentApiFirmwareIdContentFileNameHeadRequest> {
        FluentRequest {
            client: self,
            params: HeadFirmwareContentApiFirmwareIdContentFileNameHeadRequest {
                file_name: file_name.to_owned(),
                id,
            },
        }
    }
}
