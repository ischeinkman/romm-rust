use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::logout_api_logout_post`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutApiLogoutPostRequest {}
impl FluentRequest<'_, LogoutApiLogoutPostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, LogoutApiLogoutPostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/logout";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Logout

    Session logout endpoint

    Args:
        request (Request): Fastapi Request object

    Returns:
        MessageResponse: Standard message response*/
    pub fn logout_api_logout_post(&self) -> FluentRequest<'_, LogoutApiLogoutPostRequest> {
        FluentRequest {
            client: self,
            params: LogoutApiLogoutPostRequest {},
        }
    }
}
