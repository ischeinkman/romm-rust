use romm_api::*;
use serde::{de::DeserializeOwned, Deserialize};
use url::Url;

pub struct RommClient {
    client: ureq::Agent,
    auth_value: String,
    url_base: Url,
}

impl RommClient {
    pub fn new(url_base: Url, auth_value: String) -> Self {
        let client = ureq::Agent::new_with_config(
            ureq::config::Config::builder()
                .accept("application/json")
                .build(),
        );
        Self {
            client,
            url_base,
            auth_value,
        }
    }
    pub fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, anyhow::Error> {
        let n = format!(
            "{}/api/{}",
            self.url_base.as_str().trim_end_matches('/'),
            endpoint.trim_matches('/')
        );
        let s = self
            .client
            .get(n.as_str())
            .header("Authorization", &self.auth_value)
            .call()?
            .into_body()
            .read_to_string()?;
        serde_json::from_str(&s).map_err(From::from)
    }
}

fn main() {
    let url = Url::parse("https://romm.k8s.ilans.dev/").unwrap();
    let auth = format!("Basic {}", std::env::var("ROMM_API_TOKEN").unwrap());
    let cl = RommClient::new(url, auth);
    println!(
        "{:?}",
        cl.get::<Vec<PlatformSchema>>("/platforms")
            .unwrap()
    );
}
