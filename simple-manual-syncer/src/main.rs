use std::{
    env,
    time::{Duration, Instant},
};

use embedded_graphics::{
    mono_font::{iso_8859_16::FONT_10X20, MonoFont, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::{DrawTarget, Point, RgbColor},
    text::Text,
    Drawable,
};
use embedded_vintage_fonts::FONT_24X32;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter, FmtSubscriber};

mod database;
use database::SaveMetaDatabase;
mod md5hash;
mod rommclient;
use rommclient::RommClient;
mod config;
use config::Config;
mod deviceclient;
mod model;
use model::SaveMeta;
mod path_format_strings;
mod syncing;
use syncing::run_sync;
use ui::{InputReader, MiyooButton, MiyooFramebuffer};
mod ui;
mod utils;

fn main() {
    init_logger();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async_main());
}

fn init_logger() {
    let trace_env = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var("ROM_SYNC_LOG")
        .from_env()
        .unwrap();
    let mut subscriber = FmtSubscriber::builder()
        .with_env_filter(trace_env)
        .with_file(true)
        .with_line_number(true);
    let no_color = env::var_os("NO_COLOR").map_or(false, |s| !s.eq_ignore_ascii_case("0"));
    let json_log = env::var_os("ROM_SYNC_LOG_JSON").map_or(false, |s| !s.eq_ignore_ascii_case("0"));
    match (no_color, json_log) {
        (false, false) => {
            subscriber = subscriber.with_ansi(true);
        }
        (true, false) => {
            subscriber = subscriber.with_ansi(false);
        }
        (false, true) => {
            todo!()
        }
        (true, true) => {
            todo!()
        }
    }
    subscriber.finish().init();
}

#[allow(unused)]
#[tracing::instrument]
async fn async_main_1() {
    let args = std::env::args().collect::<Vec<_>>();
    let cfg = Config::load(args.into_iter().skip(1)).unwrap();
    let db = SaveMetaDatabase::open(cfg.system.database.as_deref().unwrap()).unwrap();
    info!("Starting with config: {cfg:?}");
    let cl = RommClient::new(
        cfg.romm.url.clone().unwrap(),
        cfg.romm.api_key.clone().unwrap(),
    );

    run_sync(&cfg, &cl, &db).await.unwrap();
}

async fn async_main() {
    let mut fb = MiyooFramebuffer::find_any().unwrap();
    let mut input = InputReader::new().unwrap();
    loop {
        fb.flush().unwrap();
        let (btn, evt) = input.next_event().await.unwrap();
        fb.clear(Rgb888::new(0x11, 0xee, 0x22)).unwrap();
        let msg = format!("{btn:?}, {evt:?}");

        let txt = Text::new(
            &msg,
            Point::new(60, 60),
            MonoTextStyle::new(&FONT_24X32, Rgb888::new(0xff, 0, 0xee)),
        );
        txt.draw(&mut fb).unwrap();
        if btn == MiyooButton::Start {
            break;
        }
    }
}
