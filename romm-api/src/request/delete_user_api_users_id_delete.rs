use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::delete_user_api_users_id_delete`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserApiUsersIdDeleteRequest {
    pub id: i64,
}
impl FluentRequest<'_, DeleteUserApiUsersIdDeleteRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, DeleteUserApiUsersIdDeleteRequest> {
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/api/users/{id}", id = self.params.id);
            let mut r = self.client.client.delete(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Delete User

    Delete user endpoint

    Args:
        request (Request): Fastapi Request object
        user_id (int): User internal id

    Raises:
        HTTPException: User is not found in database
        HTTPException: User deleting itself
        HTTPException: User is the last admin user

    Returns:
        MessageResponse: Standard message response*/
    pub fn delete_user_api_users_id_delete(
        &self,
        id: i64,
    ) -> FluentRequest<'_, DeleteUserApiUsersIdDeleteRequest> {
        FluentRequest {
            client: self,
            params: DeleteUserApiUsersIdDeleteRequest { id },
        }
    }
}
