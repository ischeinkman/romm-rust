use chrono::{DateTime, Utc};
use futures::stream::FuturesUnordered;
use futures::stream::TryStreamExt;
use reqwest::Client as HttpClient;
use reqwest::Error as HttpError;
use reqwest::{Body, ClientBuilder, Response};
use romm_api::{DetailedRomSchema, RomSchema, SaveSchema};
use serde::de::DeserializeOwned;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::{collections::HashMap, path::Path, sync::RwLock, time::SystemTime};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tracing::{debug, trace};
use url::Url;

use crate::{
    md5hash::{md5_stream, Md5Hash},
    SaveMeta,
};

pub struct RawClient {
    client: HttpClient,
    auth_value: String,
    url_base: Url,
}

impl RawClient {
    pub fn new(url_base: Url, auth_value: String) -> Self {
        let client = ClientBuilder::new().build().unwrap();
        Self {
            client,
            url_base,
            auth_value,
        }
    }

    pub async fn raw_put(
        &self,
        endpoint: &str,
        body: impl Into<Body>,
    ) -> Result<Response, HttpError> {
        let n = format!(
            "{}/{}",
            self.url_base.as_str().trim_end_matches('/'),
            endpoint.trim_matches('/')
        );
        trace!("Calling PUT on ROMM url {n}");
        self.client
            .put(n.as_str())
            .header("Authorization", &self.auth_value)
            .body(body)
            .send()
            .await?
            .error_for_status()
    }
    pub async fn raw_post(
        &self,
        endpoint: &str,
        body: impl Into<Body>,
    ) -> Result<Response, HttpError> {
        let n = format!(
            "{}/{}",
            self.url_base.as_str().trim_end_matches('/'),
            endpoint.trim_matches('/')
        );
        trace!("Calling POST on ROMM url {n}");
        self.client
            .post(n.as_str())
            .header("Authorization", &self.auth_value)
            .body(body)
            .send()
            .await?
            .error_for_status()
    }

    pub async fn raw_get(&self, endpoint: &str) -> Result<Response, HttpError> {
        let n = format!(
            "{}/{}",
            self.url_base.as_str().trim_end_matches('/'),
            endpoint.trim_matches('/')
        );
        trace!("Calling GET on ROMM url {n}");
        self.client
            .get(n.as_str())
            .header("Authorization", &self.auth_value)
            .send()
            .await?
            .error_for_status()
    }
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, anyhow::Error> {
        let data = self.raw_get(endpoint).await?.text().await?;
        serde_json::from_str(&data).map_err(From::from)
    }
}

pub struct RommClient {
    raw: RawClient,
    rom_id_cache: RwLock<HashMap<String, i64>>,
}

impl RommClient {
    pub fn new(url_base: Url, auth_value: String) -> Self {
        let raw = RawClient::new(url_base, auth_value);
        let rom_id_cache = RwLock::new(HashMap::new());
        Self { raw, rom_id_cache }
    }

    #[expect(unused)]
    #[tracing::instrument(skip(self))]
    pub async fn push_save(&self, save: &Path, meta: &RommSaveMeta) -> Result<(), anyhow::Error> {
        let mut fh = File::open(save).await?;
        if let Some(prev) = meta.save_id {
            let ep = format!("/api/saves/{prev}");
            self.raw.raw_put(&ep, fh).await?;
            Ok(())
        } else {
            let mut ep = format!("/api/saves?rom={}", meta.rom_id);
            if let Some(emu) = meta.meta.emulator.as_deref() {
                ep.push_str("&emulator=");
                ep.push_str(emu);
            }
            self.raw.raw_post(&ep, fh).await?;
            Ok(())
        }
    }
    #[tracing::instrument(skip(self))]
    pub async fn pull_save(&self, save: &Path, meta: &RommSaveMeta) -> Result<(), anyhow::Error> {
        let tmp_fname = save.with_extension(timestamp_now().to_rfc3339());
        let mut fh = File::create_new(&tmp_fname).await?;
        let ep = meta
            .download_path
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Download path not found."))?;
        let mut resp = self.raw.raw_get(ep).await?;
        while let Some(chunk) = resp.chunk().await? {
            fh.write_all(&chunk).await?;
        }
        fh.flush().await?;
        drop(fh);
        fs::rename(tmp_fname, save).await?;
        Ok(())
    }
    #[tracing::instrument(skip(self))]
    async fn rom_id(&self, rom: &str) -> Result<i64, anyhow::Error> {
        trace!("Resolving ROMM id for rom {rom}.");
        if let Some(id) = self
            .rom_id_cache
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(rom)
        {
            trace!("Cache hit: {id}");
            return Ok(*id);
        }
        Ok(self.rom_schema(rom).await?.id)
    }
    #[tracing::instrument(skip(self))]
    async fn rom_schema(&self, rom: &str) -> Result<RomSchema, anyhow::Error> {
        let encoded = url::form_urlencoded::byte_serialize(rom.as_bytes()).fold(
            String::new(),
            |mut acc, cur| {
                acc.push_str(cur);
                acc
            },
        );
        let mut all_found = self
            .raw
            .get::<Vec<RomSchema>>(&format!("/api/roms?search_term={encoded}"))
            .await?;
        let found = match all_found.len() {
            0 | 1 => all_found
                .pop()
                .ok_or_else(|| anyhow::anyhow!("No romm entry found for rom {rom}"))?,
            other => {
                return Err(anyhow::anyhow!(
                    "Found {other} roms for file {rom}: {all_found:?}"
                ));
            }
        };
        self.rom_id_cache
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(rom.to_owned(), found.id);
        Ok(found)
    }

    #[tracing::instrument(skip(self))]
    async fn saves_for_rom(&self, rom: &str) -> Result<Vec<RommSaveMeta>, anyhow::Error> {
        let detailed_schema = self
            .raw
            .get::<DetailedRomSchema>(&format!("/api/roms/{}", self.rom_id(rom).await?))
            .await?;
        parse_romm_saves(&self.raw, &detailed_schema)
            .await
            .map_err(From::from)
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_save_matching(&self, meta: &SaveMeta) -> Result<RommSaveMeta, anyhow::Error> {
        debug!("Looking for saves matching given metadata.");
        let all_possible = self.saves_for_rom(&meta.rom).await?;
        debug!("Found {} possible saves.", all_possible.len());
        let mut filtered = all_possible.into_iter().filter(|save| {
            if save.meta.hash == meta.hash {
                return true;
            }
            match (save.meta.emulator.as_deref(), meta.emulator.as_deref()) {
                (Some(a), Some(b)) if !a.eq_ignore_ascii_case(b) => {
                    return false;
                }
                _ => {}
            };
            save.meta.name == meta.name
        });
        let Some(found) = filtered.next() else {
            return Ok(RommSaveMeta {
                rom_id: self.rom_id(&meta.rom).await?,
                save_id: None,
                download_path: None,
                meta: SaveMeta::new_empty(
                    meta.rom.clone(),
                    meta.name.clone(),
                    meta.emulator.clone(),
                ),
            });
        };
        if filtered.next().is_some() {
            return Err(anyhow::anyhow!("Found multiple matching criteria."));
        }
        Ok(found)
    }
}

async fn parse_romm_saves(
    client: &RawClient,
    rom_data: &DetailedRomSchema,
) -> Result<Vec<RommSaveMeta>, HttpError> {
    let mut runner = FuturesUnordered::new();
    for save in rom_data.user_saves.iter() {
        let fut = async {
            let rom = rom_data.file_name_no_ext.clone();
            let name = save.file_name_no_ext.clone();
            let emulator = save.emulator.clone();
            let created = save.created_at;
            let updated = save.updated_at;
            let (hash, size) = romm_save_md5_size(client, save).await?;
            let meta = SaveMeta {
                rom,
                name,
                emulator,
                created,
                updated,
                hash,
                size,
            };
            Result::<_, HttpError>::Ok(RommSaveMeta::from_data(
                rom_data.id,
                Some(save.id),
                Some(save.download_path.clone()),
                meta,
            ))
        };
        runner.push(fut);
    }
    let mut retvl = Vec::new();
    while let Some(res) = runner.try_next().await? {
        retvl.push(res);
    }
    Ok(retvl)
}

async fn romm_save_md5_size(
    client: &RawClient,
    save: &SaveSchema,
) -> Result<(Md5Hash, u64), HttpError> {
    let raw_resp = client
        .raw_get(&save.download_path)
        .await?
        .error_for_status()?
        .bytes_stream();
    let size_counter = AtomicU64::new(0);
    let raw_resp = raw_resp.map_ok(|chunk| {
        size_counter.fetch_add(chunk.len() as _, Ordering::Release);
        chunk
    });
    let hash = md5_stream(raw_resp).await?;
    Ok((hash, size_counter.load(Ordering::Acquire)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RommSaveMeta {
    pub rom_id: i64,
    pub save_id: Option<i64>,
    pub download_path: Option<String>,
    pub meta: SaveMeta,
}

impl RommSaveMeta {
    pub fn from_data(
        rom_id: i64,
        save_id: Option<i64>,
        download_path: Option<String>,
        meta: SaveMeta,
    ) -> Self {
        Self {
            rom_id,
            download_path,
            save_id,
            meta,
        }
    }
    pub fn new_save(rom_id: i64, meta: SaveMeta) -> Self {
        Self::from_data(rom_id, None, None, meta)
    }
}

fn timestamp_now() -> DateTime<Utc> {
    let dt = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as _;
    DateTime::from_timestamp_nanos(dt)
}
