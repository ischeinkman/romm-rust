use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::add_firmware_api_firmware_post`].

On request success, this will return a [`AddFirmwareResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFirmwareApiFirmwarePostRequest {
    pub platform_id: i64,
}
impl FluentRequest<'_, AddFirmwareApiFirmwarePostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, AddFirmwareApiFirmwarePostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::AddFirmwareResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/firmware";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Add Firmware

    Upload firmware files endpoint

    Args:
        request (Request): Fastapi Request object
        platform_slug (str): Slug of the platform where to upload the files
        files (list[UploadFile], optional): List of files to upload

    Raises:
        HTTPException: No files were uploaded

    Returns:
        AddFirmwareResponse: Standard message response*/
    pub fn add_firmware_api_firmware_post(
        &self,
        platform_id: i64,
    ) -> FluentRequest<'_, AddFirmwareApiFirmwarePostRequest> {
        FluentRequest {
            client: self,
            params: AddFirmwareApiFirmwarePostRequest { platform_id },
        }
    }
}
