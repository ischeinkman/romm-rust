use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
/**You should use this struct via [`RommApiClient::delete_exclusion_api_config_exclude_exclusion_type_exclusion_value_delete`].

On request success, this will return a [`MessageResponse`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteExclusionApiConfigExcludeExclusionTypeExclusionValueDeleteRequest {
    pub exclusion_type: String,
    pub exclusion_value: String,
}
impl FluentRequest<'_, DeleteExclusionApiConfigExcludeExclusionTypeExclusionValueDeleteRequest> {}
impl<'a> ::std::future::IntoFuture
    for FluentRequest<'a, DeleteExclusionApiConfigExcludeExclusionTypeExclusionValueDeleteRequest>
{
    type Output = httpclient::InMemoryResult<crate::model::MessageResponse>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/api/config/exclude/{exclusion_type}/{exclusion_value}",
                exclusion_type = self.params.exclusion_type,
                exclusion_value = self.params.exclusion_value
            );
            let mut r = self.client.client.delete(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Delete Exclusion

    Delete platform binding from the configuration*/
    pub fn delete_exclusion_api_config_exclude_exclusion_type_exclusion_value_delete(
        &self,
        exclusion_type: &str,
        exclusion_value: &str,
    ) -> FluentRequest<'_, DeleteExclusionApiConfigExcludeExclusionTypeExclusionValueDeleteRequest>
    {
        FluentRequest {
            client: self,
            params: DeleteExclusionApiConfigExcludeExclusionTypeExclusionValueDeleteRequest {
                exclusion_type: exclusion_type.to_owned(),
                exclusion_value: exclusion_value.to_owned(),
            },
        }
    }
}
