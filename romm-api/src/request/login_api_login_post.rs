use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::login_api_login_post`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginApiLoginPostRequest {}
impl FluentRequest<'_, LoginApiLoginPostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, LoginApiLoginPostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/login";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Login

    Session login endpoint

    Args:
        request (Request): Fastapi Request object
        credentials: Defaults to Depends(HTTPBasic()).

    Raises:
        CredentialsException: Invalid credentials
        UserDisabledException: Auth is disabled

    Returns:
        MessageResponse: Standard message response*/
    pub fn login_api_login_post(&self) -> FluentRequest<'_, LoginApiLoginPostRequest> {
        FluentRequest {
            client: self,
            params: LoginApiLoginPostRequest {},
        }
    }
}
