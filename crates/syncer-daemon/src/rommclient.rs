use futures::stream;
use futures::stream::FuturesUnordered;
use futures::stream::TryStreamExt;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use reqwest::multipart::Form;
use reqwest::multipart::Part;
use reqwest::Client as HttpClient;
use reqwest::Error as HttpError;
use reqwest::{ClientBuilder, Response};
use romm_api::{DetailedRomSchema, RomSchema, SaveSchema};
use serde::de::DeserializeOwned;
use std::io;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::{collections::HashMap, path::Path, sync::RwLock};
use thiserror::Error;
use tracing::warn;
use tracing::{debug, error, info, trace};
use url::Url;

use syncer_model::path_format_strings::FormatString;

use crate::utils::download;
use crate::{
    md5hash::{md5_stream, Md5Hash},
    SaveMeta,
};

/// Wrapper struct for making raw requests to the ROMM instance.
pub struct RawClient {
    client: HttpClient,
    url_base: Url,
}

impl RawClient {
    pub fn new(url_base: Url, auth_value: String) -> Self {
        let mut default_headers = HeaderMap::new();
        let mut auth_header = HeaderValue::from_bytes(auth_value.as_bytes()).unwrap();
        auth_header.set_sensitive(true);
        default_headers.insert(HeaderName::from_static("authorization"), auth_header);
        let client = ClientBuilder::new()
            .default_headers(default_headers)
            .build()
            .unwrap();
        Self { client, url_base }
    }

    pub async fn raw_post_form(
        &self,
        endpoint: &str,
        body: impl Into<Form>,
    ) -> Result<Response, HttpError> {
        let n = format!(
            "{}/{}",
            self.url_base.as_str().trim_end_matches('/'),
            endpoint.trim_matches('/')
        );
        trace!("Calling POST (with form) on ROMM url {n}");
        let req = self.client.post(n.as_str()).multipart(body.into());
        req.send().await?.error_for_status()
    }

    pub async fn raw_get(&self, endpoint: &str) -> Result<Response, HttpError> {
        let n = format!(
            "{}/{}",
            self.url_base.as_str().trim_end_matches('/'),
            endpoint.trim_matches('/')
        );
        trace!("Calling GET on ROMM url {n}");
        self.client.get(n.as_str()).send().await?.error_for_status()
    }
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, RommError> {
        let data = self.raw_get(endpoint).await?.text().await?;
        serde_json::from_str(&data).map_err(From::from)
    }
}

pub struct RommClient {
    raw: RawClient,
    /// Cache of rom name to ROMM ID for quick lookup.
    rom_id_cache: RwLock<HashMap<String, i64>>,
}

impl RommClient {
    pub fn new(url_base: Url, auth_value: String) -> Self {
        let raw = RawClient::new(url_base, auth_value);
        let rom_id_cache = RwLock::new(HashMap::new());
        Self { raw, rom_id_cache }
    }

    #[tracing::instrument(skip(self))]
    pub async fn push_save(
        &self,
        save: &Path,
        meta: &RommSaveMeta,
        fmt: Option<&FormatString>,
    ) -> Result<(), RommError> {
        info!(
            "Pushing ROMM save to rom {} from local path {}.",
            meta.rom_id,
            save.display()
        );
        if meta.meta.timestamp().timestamp() <= 60 * 60 * 24 * 30 {
            warn!("Found too low timestamp before pushing save: {meta:?}");
            return Ok(());
        }
        let mut ep = format!("/api/saves?rom_id={}", meta.rom_id);
        if let Some(emu) = meta.meta.emulator.as_deref() {
            ep.push_str("&emulator=");
            ep.push_str(emu);
        }

        let target = fmt
            .map(|fmt| meta.meta.output_target(fmt))
            .unwrap_or_else(|| format!("{}.{}", meta.meta.name, meta.meta.ext));

        let part = Part::file(save).await?.file_name(target);
        debug!("Pushing file to remote: {part:?}");
        let form = Form::new().part("saves", part);
        self.raw.raw_post_form(&ep, form).await?;
        info!("Finished save upload.");
        Ok(())
    }
    #[tracing::instrument(skip(self))]
    pub async fn pull_save(&self, save: &Path, meta: &RommSaveMeta) -> Result<(), anyhow::Error> {
        let ep = meta
            .download_path
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Download path not found."))?;
        info!(
            "Pulling ROMM save {}/{} to local path {}.",
            meta.rom_id,
            meta.save_id.unwrap(),
            save.display()
        );
        debug!("Starting download: {save:?} {meta:?}");

        let dl_stream =
            stream::try_unfold(self.raw.raw_get(ep).await?, move |mut resp| async move {
                match resp.chunk().await {
                    Err(e) => Err(e),
                    Ok(None) => Ok(None),
                    Ok(Some(chunk)) => Ok(Some((chunk, resp))),
                }
            });
        download(dl_stream, save).await?;
        info!("Finished ROMM save.");
        Ok(())
    }
    #[tracing::instrument(skip(self))]
    async fn rom_id(&self, rom: &str) -> Result<i64, RommError> {
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
    async fn rom_schema(&self, rom: &str) -> Result<RomSchema, RommError> {
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
                .ok_or_else(|| RommError::RomNotFound(rom.to_owned()))?,
            other => {
                return Err(RommError::TooManyRoms {
                    rom: rom.to_owned(),
                    count: other,
                });
            }
        };
        self.rom_id_cache
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(rom.to_owned(), found.id);
        Ok(found)
    }

    #[tracing::instrument(skip(self))]
    async fn saves_for_rom(&self, rom: &str) -> Result<Vec<RommSaveMeta>, RommError> {
        let detailed_schema = self
            .raw
            .get::<DetailedRomSchema>(&format!("/api/roms/{}", self.rom_id(rom).await?))
            .await?;
        parse_romm_saves(&self.raw, &detailed_schema)
            .await
            .map_err(From::from)
    }

    /// Finds a save in the ROMM database matching the given [`SaveMeta`] record and [`FormatString`].
    ///
    /// If multiple are found, returns the latest.
    ///
    /// # Errors
    /// * No rom matched the [`SaveMeta`] record's rom string.
    /// * More than 1 rom matched the [`SaveMeta`] record's rom string.
    /// * Multiple saves match the latest timestamp.
    /// * There was an error communicated with the remote ROMM server.
    #[tracing::instrument(skip(self))]
    pub async fn find_save_matching(
        &self,
        meta: &SaveMeta,
        fmt: Option<&FormatString>,
    ) -> Result<RommSaveMeta, RommError> {
        debug!("Looking for saves matching given metadata.");
        let all_possible = self.saves_for_rom(meta.rom()).await?;
        debug!("Found {} possible saves.", all_possible.len());
        let filtered = all_possible.into_iter().filter(|save| {
            match (fmt, save.raw_name.as_deref()) {
                (Some(fmt), Some(name)) if !fmt.matches(name) => {
                    return false;
                }
                (Some(_), None) => {
                    error!("Found a remote ROMM save whose raw_name we couldn't apply!? This should be impossible. Meta is: {save:?}");
                    return false;
                }
                _ => {}
            }
            if save.meta.same_file(meta) {
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
        let mut all_found: Vec<RommSaveMeta> = Vec::new();
        for found in filtered {
            let cur_ts = all_found.first().map(|meta| meta.meta.timestamp());
            match (cur_ts, found.meta.timestamp()) {
                (None, _) => {
                    all_found.push(found);
                }
                (Some(cur_ts), nxt_ts) if cur_ts == nxt_ts => {
                    all_found.push(found);
                }
                (Some(cur_ts), nxt_ts) if cur_ts < nxt_ts => {
                    all_found.clear();
                    all_found.push(found);
                }
                (Some(_), _) => {}
            }
        }
        match all_found.len() {
            0 => {
                let rom_id = self.rom_id(meta.rom()).await?;
                Ok(RommSaveMeta::new_save(rom_id, meta))
            }
            1 => Ok(all_found.pop().unwrap()),
            count => Err(RommError::TooManySaves {
                meta: meta.clone(),
                count,
            }),
        }
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
            let raw_name = save.file_name.clone();
            let name = save.file_name_no_ext.clone();
            let ext = save.file_extension.clone();
            let emulator = save.emulator.clone();
            let created = save.created_at;
            let updated = save.updated_at;
            let (hash, size) = romm_save_md5_size(client, save).await?;
            let meta = SaveMeta {
                rom: Some(rom),
                name,
                ext,
                emulator,
                created,
                updated,
                hash,
                size,
            };
            Result::<_, HttpError>::Ok(RommSaveMeta::from_data(
                Some(raw_name),
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
    pub raw_name: Option<String>,
    pub meta: SaveMeta,
}

impl RommSaveMeta {
    pub fn from_data(
        raw_name: Option<String>,
        rom_id: i64,
        save_id: Option<i64>,
        download_path: Option<String>,
        meta: SaveMeta,
    ) -> Self {
        Self {
            raw_name,
            rom_id,
            download_path,
            save_id,
            meta,
        }
    }
    pub fn new_save(rom_id: i64, base_meta: &SaveMeta) -> Self {
        let meta = SaveMeta::new_empty(
            base_meta.rom().to_owned(),
            base_meta.name.clone(),
            base_meta.ext.clone(),
            base_meta.emulator.clone(),
        );
        Self::from_data(None, rom_id, None, None, meta)
    }
}

#[derive(Debug, Error)]
pub enum RommError {
    #[error("No rom found with name {0}")]
    RomNotFound(String),
    #[error("Found {count} possible roms matching term {rom}")]
    TooManyRoms { rom: String, count: usize },
    #[error("Found {count} possible saves matching filter {meta:?}")]
    TooManySaves { meta: SaveMeta, count: usize },
    #[error(transparent)]
    JsonParser(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Http(#[from] HttpError),
}
