use crate::FluentRequest;
use httpclient::InMemoryResponseExt;
use serde::{Deserialize, Serialize};
use crate::model::*;
/**You should use this struct via [`RommApiClient::search_cover_api_search_cover_get`].

On request success, this will return a [`Vec<SearchCoverSchema>`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCoverApiSearchCoverGetRequest {
    pub search_term: Option<String>,
}
impl FluentRequest<'_, SearchCoverApiSearchCoverGetRequest> {
    ///Set the value of the search_term field.
    pub fn search_term(mut self, search_term: &str) -> Self {
        self.params.search_term = Some(search_term.to_owned());
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, SearchCoverApiSearchCoverGetRequest> {
    type Output = httpclient::InMemoryResult<crate::model::Vec<SearchCoverSchema>>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/api/search/cover";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}
impl crate::RommApiClient {
    ///Search Cover
    pub fn search_cover_api_search_cover_get(
        &self,
    ) -> FluentRequest<'_, SearchCoverApiSearchCoverGetRequest> {
        FluentRequest {
            client: self,
            params: SearchCoverApiSearchCoverGetRequest { search_term: None },
        }
    }
}
