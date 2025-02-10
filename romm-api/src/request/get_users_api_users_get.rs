use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
use crate::model::*;
/**You should use this struct via [`RommApiClient::get_users_api_users_get`].

On request success, this will return a [`Vec<UserSchema>`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUsersApiUsersGetRequest {}
impl FluentRequest<'_, GetUsersApiUsersGetRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetUsersApiUsersGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::Vec<UserSchema>>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/users";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Users

    Get all users endpoint

    Args:
        request (Request): Fastapi Request object

    Returns:
        list[UserSchema]: All users stored in the RomM's database*/
    pub fn get_users_api_users_get(&self) -> FluentRequest<'_, GetUsersApiUsersGetRequest> {
        FluentRequest {
            client: self,
            params: GetUsersApiUsersGetRequest {},
        }
    }
}
