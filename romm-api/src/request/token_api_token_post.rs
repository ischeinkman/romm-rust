use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::token_api_token_post`].

On request success, this will return a [`TokenResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenApiTokenPostRequest {}
impl FluentRequest<'_, TokenApiTokenPostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, TokenApiTokenPostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::TokenResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/token";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Token

    OAuth2 token endpoint

    Args:
        form_data (Annotated[OAuth2RequestForm, Depends): Form Data with OAuth2 info

    Raises:
        HTTPException: Missing refresh token
        HTTPException: Invalid refresh token
        HTTPException: Missing username or password
        HTTPException: Invalid username or password
        HTTPException: Client credentials are not yet supported
        HTTPException: Invalid or unsupported grant type
        HTTPException: Insufficient scope

    Returns:
        TokenResponse: TypedDict with the new generated token info*/
    pub fn token_api_token_post(&self) -> FluentRequest<'_, TokenApiTokenPostRequest> {
        FluentRequest {
            client: self,
            params: TokenApiTokenPostRequest {},
        }
    }
}
