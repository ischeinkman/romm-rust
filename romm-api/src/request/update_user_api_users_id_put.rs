use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::update_user_api_users_id_put`].

On request success, this will return a [`UserSchema`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserApiUsersIdPutRequest {
    pub email: Option<serde_json::Value>,
    pub enabled: Option<serde_json::Value>,
    pub id: i64,
    pub password: Option<serde_json::Value>,
    pub role: Option<serde_json::Value>,
    pub username: Option<serde_json::Value>,
}
impl FluentRequest<'_, UpdateUserApiUsersIdPutRequest> {
    ///Set the value of the email field.
    pub fn email(mut self, email: serde_json::Value) -> Self {
        self.params.email = Some(email);
        self
    }
    ///Set the value of the enabled field.
    pub fn enabled(mut self, enabled: serde_json::Value) -> Self {
        self.params.enabled = Some(enabled);
        self
    }
    ///Set the value of the password field.
    pub fn password(mut self, password: serde_json::Value) -> Self {
        self.params.password = Some(password);
        self
    }
    ///Set the value of the role field.
    pub fn role(mut self, role: serde_json::Value) -> Self {
        self.params.role = Some(role);
        self
    }
    ///Set the value of the username field.
    pub fn username(mut self, username: serde_json::Value) -> Self {
        self.params.username = Some(username);
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, UpdateUserApiUsersIdPutRequest> {
    type Output = httpclient::InMemoryResult<crate::model::UserSchema>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/users/{id}", id = self.params.id);
            let mut r = self.client.client.put(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Update User

    Update user endpoint

    Args:
        request (Request): Fastapi Requests object
        user_id (int): User internal id
        form_data (Annotated[UserUpdateForm, Depends): Form Data with user updated info

    Raises:
        HTTPException: User is not found in database
        HTTPException: Username already in use by another user

    Returns:
        UserSchema: Updated user info*/
    pub fn update_user_api_users_id_put(
        &self,
        id: i64,
    ) -> FluentRequest<'_, UpdateUserApiUsersIdPutRequest> {
        FluentRequest {
            client: self,
            params: UpdateUserApiUsersIdPutRequest {
                email: None,
                enabled: None,
                id,
                password: None,
                role: None,
                username: None,
            },
        }
    }
}
