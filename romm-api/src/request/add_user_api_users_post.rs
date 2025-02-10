use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::add_user_api_users_post`].

On request success, this will return a [`UserSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddUserApiUsersPostRequest {
    pub email: String,
    pub password: String,
    pub role: String,
    pub username: String,
}
pub struct AddUserApiUsersPostRequired<'a> {
    pub email: &'a str,
    pub password: &'a str,
    pub role: &'a str,
    pub username: &'a str,
}
impl FluentRequest<'_, AddUserApiUsersPostRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, AddUserApiUsersPostRequest> {
    type Output = httpclient::InMemoryResult<crate::model::UserSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/users";
            let mut r = self.client.client.post(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Add User

    Create user endpoint

    Args:
        request (Request): Fastapi Requests object
        username (str): User username
        password (str): User password
        email (str): User email
        role (str): RomM Role object represented as string

    Returns:
        UserSchema: Newly created user*/
    pub fn add_user_api_users_post(
        &self,
        args: AddUserApiUsersPostRequired,
    ) -> FluentRequest<'_, AddUserApiUsersPostRequest> {
        FluentRequest {
            client: self,
            params: AddUserApiUsersPostRequest {
                email: args.email.to_owned(),
                password: args.password.to_owned(),
                role: args.role.to_owned(),
                username: args.username.to_owned(),
            },
        }
    }
}
