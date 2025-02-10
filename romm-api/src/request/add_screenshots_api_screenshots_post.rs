use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::add_screenshots_api_screenshots_post`].

On request success, this will return a [`UploadedScreenshotsResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddScreenshotsApiScreenshotsPostRequest {
    pub rom_id: i64,
}
impl FluentRequest<'_, AddScreenshotsApiScreenshotsPostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, AddScreenshotsApiScreenshotsPostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::UploadedScreenshotsResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/screenshots";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    ///Add Screenshots
    pub fn add_screenshots_api_screenshots_post(
        &self,
        rom_id: i64,
    ) -> FluentRequest<'_, AddScreenshotsApiScreenshotsPostRequest> {
        FluentRequest {
            client: self,
            params: AddScreenshotsApiScreenshotsPostRequest { rom_id },
        }
    }
}
