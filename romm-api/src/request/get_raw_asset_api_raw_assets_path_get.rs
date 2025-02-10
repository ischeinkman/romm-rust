use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::get_raw_asset_api_raw_assets_path_get`].

On request success, this will return a [`GetRawAssetApiRawAssetsPathGetResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRawAssetApiRawAssetsPathGetRequest {
    pub path: String,
}
impl FluentRequest<'_, GetRawAssetApiRawAssetsPathGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetRawAssetApiRawAssetsPathGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::GetRawAssetApiRawAssetsPathGetResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/raw/assets/{path}", path = self.params.path);
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Raw Asset

    Download a single asset file

    Args:
        request (Request): Fastapi Request object

    Returns:
        FileResponse: Returns a single asset file*/
    pub fn get_raw_asset_api_raw_assets_path_get(
        &self,
        path: &str,
    ) -> FluentRequest<'_, GetRawAssetApiRawAssetsPathGetRequest> {
        FluentRequest {
            client: self,
            params: GetRawAssetApiRawAssetsPathGetRequest {
                path: path.to_owned(),
            },
        }
    }
}
