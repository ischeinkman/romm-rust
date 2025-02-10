use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
use crate::model::*;
/**You should use this struct via [`RommApiClient::get_roms_api_roms_get`].

On request success, this will return a [`Vec<SimpleRomSchema>`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRomsApiRomsGetRequest {
    pub collection_id: Option<serde_json::Value>,
    pub limit: Option<serde_json::Value>,
    pub offset: Option<serde_json::Value>,
    pub order_by: Option<String>,
    pub order_dir: Option<String>,
    pub platform_id: Option<serde_json::Value>,
    pub search_term: Option<String>,
}
impl FluentRequest<'_, GetRomsApiRomsGetRequest> {
    ///Set the value of the collection_id field.
    pub fn collection_id(mut self, collection_id: serde_json::Value) -> Self {
        self.params.collection_id = Some(collection_id);
        self
    }
    ///Set the value of the limit field.
    pub fn limit(mut self, limit: serde_json::Value) -> Self {
        self.params.limit = Some(limit);
        self
    }
    ///Set the value of the offset field.
    pub fn offset(mut self, offset: serde_json::Value) -> Self {
        self.params.offset = Some(offset);
        self
    }
    ///Set the value of the order_by field.
    pub fn order_by(mut self, order_by: &str) -> Self {
        self.params.order_by = Some(order_by.to_owned());
        self
    }
    ///Set the value of the order_dir field.
    pub fn order_dir(mut self, order_dir: &str) -> Self {
        self.params.order_dir = Some(order_dir.to_owned());
        self
    }
    ///Set the value of the platform_id field.
    pub fn platform_id(mut self, platform_id: serde_json::Value) -> Self {
        self.params.platform_id = Some(platform_id);
        self
    }
    ///Set the value of the search_term field.
    pub fn search_term(mut self, search_term: &str) -> Self {
        self.params.search_term = Some(search_term.to_owned());
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, GetRomsApiRomsGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::Vec<SimpleRomSchema>>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/roms";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Get Roms

    Get roms endpoint

    Args:
        request (Request): Fastapi Request object
        platform_id (int, optional): Platform ID to filter ROMs
        collection_id (int, optional): Collection ID to filter ROMs
        search_term (str, optional): Search term to filter ROMs
        limit (int, optional): Limit the number of ROMs returned
        offset (int, optional): Offset for pagination
        order_by (str, optional): Field to order ROMs by
        order_dir (str, optional): Direction to order ROMs (asc or desc)
        last_played (bool, optional): Flag to filter ROMs by last played

    Returns:
        list[DetailedRomSchema]: List of ROMs stored in the database*/
    pub fn get_roms_api_roms_get(&self) -> FluentRequest<'_, GetRomsApiRomsGetRequest> {
        FluentRequest {
            client: self,
            params: GetRomsApiRomsGetRequest {
                collection_id: None,
                limit: None,
                offset: None,
                order_by: None,
                order_dir: None,
                platform_id: None,
                search_term: None,
            },
        }
    }
}
