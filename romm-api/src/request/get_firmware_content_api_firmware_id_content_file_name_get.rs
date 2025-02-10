use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::get_firmware_content_api_firmware_id_content_file_name_get`].

On request success, this will return a [`GetFirmwareContentApiFirmwareIdContentFileNameGetResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFirmwareContentApiFirmwareIdContentFileNameGetRequest {
    pub file_name: String,
    pub id: i64,
}
impl FluentRequest<'_, GetFirmwareContentApiFirmwareIdContentFileNameGetRequest> {}
impl<'a> ::std::future::IntoFuture
    for FluentRequest<'a, GetFirmwareContentApiFirmwareIdContentFileNameGetRequest>
{
    type Output = httpclient::InMemoryResult<
        crate::model::GetFirmwareContentApiFirmwareIdContentFileNameGetResponse,
    >;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/api/firmware/{id}/content/{file_name}",
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
    /**Get Firmware Content

    Download firmware endpoint

    Args:
        request (Request): Fastapi Request object
        id (int): Rom internal id
        file_name (str): Required due to a bug in emulatorjs

    Returns:
        FileResponse: Returns the firmware file*/
    pub fn get_firmware_content_api_firmware_id_content_file_name_get(
        &self,
        file_name: &str,
        id: i64,
    ) -> FluentRequest<'_, GetFirmwareContentApiFirmwareIdContentFileNameGetRequest> {
        FluentRequest {
            client: self,
            params: GetFirmwareContentApiFirmwareIdContentFileNameGetRequest {
                file_name: file_name.to_owned(),
                id,
            },
        }
    }
}
