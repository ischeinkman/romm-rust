use chrono::{DateTime, Utc};
use futures::{
    stream::{self, StreamExt, TryStreamExt},
    TryStream,
};
use reqwest::Client as HttpClient;
use reqwest::Error as HttpError;
use reqwest::{Body, ClientBuilder, Response};
use romm_api::{DetailedRomSchema, RomSchema, SaveSchema};
use serde::de::DeserializeOwned;
use std::{
    collections::HashMap,
    io::{self, Read},
    path::Path,
    sync::RwLock,
    task::Poll,
    time::SystemTime,
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
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
    #[expect(unused)]
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
    #[expect(unused)]
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
    async fn rom_id(&self, rom: &str) -> Result<i64, anyhow::Error> {
        if let Some(id) = self
            .rom_id_cache
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(rom)
        {
            return Ok(*id);
        }
        Ok(self.rom_schema(rom).await?.id)
    }
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
    #[expect(unused)]
    pub async fn saves_for_rom(&self, rom: &str) -> Result<Vec<RommSaveMeta>, anyhow::Error> {
        let detailed_schema = self
            .raw
            .get::<DetailedRomSchema>(&format!("/api/roms/{}", self.rom_id(rom).await?))
            .await?;
        parse_romm_saves(&self.raw, &detailed_schema)
            .await
            .map_err(From::from)
    }
}

async fn parse_romm_saves(
    client: &RawClient,
    rom_data: &DetailedRomSchema,
) -> Result<Vec<RommSaveMeta>, HttpError> {
    stream::iter(rom_data.user_saves.iter())
        .then(|save| async {
            let rom = rom_data.file_name.clone();
            let name = save.file_name.clone();
            let emulator = save.emulator.clone();
            let created = save.created_at;
            let updated = save.updated_at;
            let md5 = romm_save_md5(client, save).await?;
            let meta = SaveMeta {
                rom,
                name,
                emulator,
                created,
                updated,
                md5,
            };
            Result::<_, HttpError>::Ok(RommSaveMeta::from_data(
                rom_data.id,
                Some(save.id),
                Some(save.download_path.clone()),
                meta,
            ))
        })
        .try_collect()
        .await
}

async fn romm_save_md5(client: &RawClient, save: &SaveSchema) -> Result<Md5Hash, HttpError> {
    let raw_resp = client
        .raw_get(&save.download_path)
        .await?
        .error_for_status()?
        .bytes_stream();
    md5_stream(raw_resp).await.map_err(From::from)
}

fn wrap_reader(mut rdr: impl io::Read) -> impl TryStream<Ok = Box<[u8]>, Error = io::Error> {
    futures::stream::poll_fn(move |_| {
        let mut buff = vec![0; 4 * 1024 * 1024];
        match rdr.read(&mut buff) {
            Ok(0) => Poll::Ready(None),
            Ok(n) => {
                buff.resize(n, 0);
                Poll::Ready(Some(Ok(buff.into_boxed_slice())))
            }
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    })
}

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
