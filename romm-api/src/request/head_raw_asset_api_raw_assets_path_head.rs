use crate::FluentRequest;
use httpclient::{InMemoryResponseExt, Method};
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::head_raw_asset_api_raw_assets_path_head`].

On request success, this will return a [`HeadRawAssetApiRawAssetsPathHeadResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadRawAssetApiRawAssetsPathHeadRequest {
    pub path: String,
}
impl FluentRequest<'_, HeadRawAssetApiRawAssetsPathHeadRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, HeadRawAssetApiRawAssetsPathHeadRequest> {
    type Output =
        httpclient::InMemoryResult<crate::model::HeadRawAssetApiRawAssetsPathHeadResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/raw/assets/{path}", path = self.params.path);
            let mut r = self.client.client.request(Method::HEAD, url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    ///Head Raw Asset
    pub fn head_raw_asset_api_raw_assets_path_head(
        &self,
        path: &str,
    ) -> FluentRequest<'_, HeadRawAssetApiRawAssetsPathHeadRequest> {
        FluentRequest {
            client: self,
            params: HeadRawAssetApiRawAssetsPathHeadRequest {
                path: path.to_owned(),
            },
        }
    }
}
