use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::get_user_api_users_id_get`].

On request success, this will return a [`UserSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserApiUsersIdGetRequest {
    pub id: i64,
}
impl FluentRequest<'_, GetUserApiUsersIdGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetUserApiUsersIdGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::UserSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/users/{id}", id = self.params.id);
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get User

    Get user endpoint

    Args:
        request (Request): Fastapi Request object

    Returns:
        UserSchem: User stored in the RomM's database*/
    pub fn get_user_api_users_id_get(
        &self,
        id: i64,
    ) -> FluentRequest<'_, GetUserApiUsersIdGetRequest> {
        FluentRequest {
            client: self,
            params: GetUserApiUsersIdGetRequest { id },
        }
    }
}
