use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::login_via_openid_api_login_openid_get`].

On request success, this will return a [`LoginViaOpenidApiLoginOpenidGetResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginViaOpenidApiLoginOpenidGetRequest {}
impl FluentRequest<'_, LoginViaOpenidApiLoginOpenidGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, LoginViaOpenidApiLoginOpenidGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::LoginViaOpenidApiLoginOpenidGetResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/login/openid";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Login Via Openid

    OIDC login endpoint

    Args:
        request (Request): Fastapi Request object

    Raises:
        OIDCDisabledException: OAuth is disabled
        OIDCNotConfiguredException: OAuth not configured

    Returns:
        RedirectResponse: Redirect to OIDC provider*/
    pub fn login_via_openid_api_login_openid_get(
        &self,
    ) -> FluentRequest<'_, LoginViaOpenidApiLoginOpenidGetRequest> {
        FluentRequest {
            client: self,
            params: LoginViaOpenidApiLoginOpenidGetRequest {},
        }
    }
}
