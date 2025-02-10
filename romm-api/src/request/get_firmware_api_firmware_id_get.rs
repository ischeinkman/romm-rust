use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::get_firmware_api_firmware_id_get`].

On request success, this will return a [`FirmwareSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFirmwareApiFirmwareIdGetRequest {
    pub id: i64,
}
impl FluentRequest<'_, GetFirmwareApiFirmwareIdGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetFirmwareApiFirmwareIdGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::FirmwareSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/firmware/{id}", id = self.params.id);
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Firmware

    Get firmware endpoint

    Args:
        request (Request): Fastapi Request object
        id (int): Firmware internal id

    Returns:
        FirmwareSchema: Firmware stored in the database*/
    pub fn get_firmware_api_firmware_id_get(
        &self,
        id: i64,
    ) -> FluentRequest<'_, GetFirmwareApiFirmwareIdGetRequest> {
        FluentRequest {
            client: self,
            params: GetFirmwareApiFirmwareIdGetRequest { id },
        }
    }
}
