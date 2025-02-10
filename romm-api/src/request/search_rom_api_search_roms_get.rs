use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
use crate::model::*;
/**You should use this struct via [`RommApiClient::search_rom_api_search_roms_get`].

On request success, this will return a [`Vec<SearchRomSchema>`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRomApiSearchRomsGetRequest {
    pub rom_id: i64,
    pub search_by: Option<String>,
    pub search_term: Option<serde_json::Value>,
}
impl FluentRequest<'_, SearchRomApiSearchRomsGetRequest> {
    ///Set the value of the search_by field.
    pub fn search_by(mut self, search_by: &str) -> Self {
        self.params.search_by = Some(search_by.to_owned());
        self
    }
    ///Set the value of the search_term field.
    pub fn search_term(mut self, search_term: serde_json::Value) -> Self {
        self.params.search_term = Some(search_term);
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, SearchRomApiSearchRomsGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::Vec<SearchRomSchema>>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/search/roms";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    /**Search Rom

    Search for rom in metadata providers

    Args:
        request (Request): FastAPI request
        rom_id (int): Rom ID
        source (str): Source of the rom
        search_term (str, optional): Search term. Defaults to None.
        search_by (str, optional): Search by name or ID. Defaults to "name".
        search_extended (bool, optional): Search extended info. Defaults to False.

    Returns:
        list[SearchRomSchema]: List of matched roms*/
    pub fn search_rom_api_search_roms_get(
        &self,
        rom_id: i64,
    ) -> FluentRequest<'_, SearchRomApiSearchRomsGetRequest> {
        FluentRequest {
            client: self,
            params: SearchRomApiSearchRomsGetRequest {
                rom_id,
                search_by: None,
                search_term: None,
            },
        }
    }
}
