use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::get_current_user_api_users_me_get`].

On request success, this will return a [`GetCurrentUserApiUsersMeGetResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCurrentUserApiUsersMeGetRequest {}
impl FluentRequest<'_, GetCurrentUserApiUsersMeGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetCurrentUserApiUsersMeGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::GetCurrentUserApiUsersMeGetResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/users/me";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Current User

    Get current user endpoint

    Args:
        request (Request): Fastapi Request object

    Returns:
        UserSchema | None: Current user*/
    pub fn get_current_user_api_users_me_get(
        &self,
    ) -> FluentRequest<'_, GetCurrentUserApiUsersMeGetRequest> {
        FluentRequest {
            client: self,
            params: GetCurrentUserApiUsersMeGetRequest {},
        }
    }
}
