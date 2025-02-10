use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::auth_openid_api_oauth_openid_get`].

On request success, this will return a [`AuthOpenidApiOauthOpenidGetResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthOpenidApiOauthOpenidGetRequest {}
impl FluentRequest<'_, AuthOpenidApiOauthOpenidGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, AuthOpenidApiOauthOpenidGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::AuthOpenidApiOauthOpenidGetResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/oauth/openid";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Auth Openid

    OIDC callback endpoint

    Args:
        request (Request): Fastapi Request object

    Raises:
        OIDCDisabledException: OAuth is disabled
        OIDCNotConfiguredException: OAuth not configured
        AuthCredentialsException: Invalid credentials
        UserDisabledException: Auth is disabled

    Returns:
        RedirectResponse: Redirect to home page*/
    pub fn auth_openid_api_oauth_openid_get(
        &self,
    ) -> FluentRequest<'_, AuthOpenidApiOauthOpenidGetRequest> {
        FluentRequest {
            client: self,
            params: AuthOpenidApiOauthOpenidGetRequest {},
        }
    }
}
