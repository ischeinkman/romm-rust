use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::delete_roms_api_roms_delete_post`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRomsApiRomsDeletePostRequest {}
impl FluentRequest<'_, DeleteRomsApiRomsDeletePostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, DeleteRomsApiRomsDeletePostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/roms/delete";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Delete Roms

    Delete roms endpoint

    Args:
        request (Request): Fastapi Request object.
            {
                "roms": List of rom's ids to delete
            }
        delete_from_fs (bool, optional): Flag to delete rom from filesystem. Defaults to False.

    Returns:
        MessageResponse: Standard message response*/
    pub fn delete_roms_api_roms_delete_post(
        &self,
    ) -> FluentRequest<'_, DeleteRomsApiRomsDeletePostRequest> {
        FluentRequest {
            client: self,
            params: DeleteRomsApiRomsDeletePostRequest {},
        }
    }
}
