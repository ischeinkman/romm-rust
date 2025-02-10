use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
use crate::model::*;
/**You should use this struct via [`RommApiClient::get_platform_firmware_api_firmware_get`].

On request success, this will return a [`Vec<FirmwareSchema>`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPlatformFirmwareApiFirmwareGetRequest {
    pub platform_id: Option<serde_json::Value>,
}
impl FluentRequest<'_, GetPlatformFirmwareApiFirmwareGetRequest> {
    ///Set the value of the platform_id field.
    pub fn platform_id(mut self, platform_id: serde_json::Value) -> Self {
        self.params.platform_id = Some(platform_id);
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetPlatformFirmwareApiFirmwareGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::Vec<FirmwareSchema>>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/firmware";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Platform Firmware

    Get firmware endpoint

    Args:
        request (Request): Fastapi Request object

    Returns:
        list[FirmwareSchema]: Firmware stored in the database*/
    pub fn get_platform_firmware_api_firmware_get(
        &self,
    ) -> FluentRequest<'_, GetPlatformFirmwareApiFirmwareGetRequest> {
        FluentRequest {
            client: self,
            params: GetPlatformFirmwareApiFirmwareGetRequest { platform_id: None },
        }
    }
}
